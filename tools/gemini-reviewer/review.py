#!/usr/bin/env python3
"""
Custom Gemini PR Reviewer — runs as a GitHub Action step.

Uses Gemini 3.1 Pro (Google's most powerful model as of March 2026, #1 on
12/18 benchmarks, 1M token context) to review pull requests with FULL context:
the entire source tree, issue details, PR comments, specs, and coding standards.

Environment variables required:
  GEMINI_API_KEY    — Google AI API key (from GCP project)
  GITHUB_TOKEN      — GitHub token with pull-requests:write scope
  PR_NUMBER         — Pull request number to review
  GITHUB_REPOSITORY — owner/repo (e.g., bedwards/copper-hollow)
"""

import json
import os
import re
import subprocess
import sys
from pathlib import Path


from google import genai
from google.genai import types


GEMINI_MODEL = "gemini-3.1-pro"  # Most powerful model, March 2026
GITHUB_REPO = os.environ.get("GITHUB_REPOSITORY", "bedwards/copper-hollow")


def gh_api(endpoint: str, extra_args: list[str] | None = None,
           timeout: int = 60) -> str:
    """Call gh api and return stdout."""
    cmd = ["gh", "api", endpoint]
    if extra_args:
        cmd.extend(extra_args)
    result = subprocess.run(
        cmd, capture_output=True, text=True, timeout=timeout,
    )
    if result.returncode != 0:
        raise RuntimeError(f"gh api {endpoint} failed: {result.stderr}")
    return result.stdout


def get_pr_info(pr_number: int) -> dict:
    """Get PR metadata (title, body, user, labels, etc.)."""
    return json.loads(gh_api(f"repos/{GITHUB_REPO}/pulls/{pr_number}"))


def get_pr_diff(pr_number: int) -> str:
    """Get the full unified diff for a PR."""
    return gh_api(
        f"repos/{GITHUB_REPO}/pulls/{pr_number}",
        ["-H", "Accept: application/vnd.github.diff"],
    )


def get_pr_comments(pr_number: int) -> list[dict]:
    """Get all review comments and issue comments on the PR."""
    comments = []

    # PR review comments (inline)
    try:
        data = json.loads(gh_api(
            f"repos/{GITHUB_REPO}/pulls/{pr_number}/comments",
            ["--paginate"],
        ))
        for c in data:
            comments.append({
                "user": c.get("user", {}).get("login", "unknown"),
                "path": c.get("path", ""),
                "line": c.get("line"),
                "body": c.get("body", ""),
                "type": "inline",
            })
    except Exception:
        pass

    # Issue comments (conversation)
    try:
        data = json.loads(gh_api(
            f"repos/{GITHUB_REPO}/issues/{pr_number}/comments",
            ["--paginate"],
        ))
        for c in data:
            comments.append({
                "user": c.get("user", {}).get("login", "unknown"),
                "body": c.get("body", ""),
                "type": "conversation",
            })
    except Exception:
        pass

    # PR reviews (summary reviews)
    try:
        data = json.loads(gh_api(
            f"repos/{GITHUB_REPO}/pulls/{pr_number}/reviews",
        ))
        for r in data:
            if r.get("body"):
                comments.append({
                    "user": r.get("user", {}).get("login", "unknown"),
                    "state": r.get("state", ""),
                    "body": r.get("body", ""),
                    "type": "review",
                })
    except Exception:
        pass

    return comments


def get_linked_issue(pr_info: dict) -> dict | None:
    """Extract and fetch the linked issue from PR title/body."""
    body = (pr_info.get("body") or "") + " " + (pr_info.get("title") or "")

    # Match patterns like "closes #123", "fixes #45", "(#67)"
    matches = re.findall(r'(?:closes?|fixes?|resolves?)\s+#(\d+)|#(\d+)',
                         body, re.IGNORECASE)
    issue_numbers = set()
    for m in matches:
        num = m[0] or m[1]
        if num:
            issue_numbers.add(int(num))

    if not issue_numbers:
        return None

    # Fetch the first linked issue
    issue_num = min(issue_numbers)
    try:
        data = json.loads(gh_api(f"repos/{GITHUB_REPO}/issues/{issue_num}"))
        return {
            "number": issue_num,
            "title": data.get("title", ""),
            "body": data.get("body", ""),
            "labels": [l.get("name") for l in data.get("labels", [])],
        }
    except Exception:
        return None


def get_changed_files_list(pr_number: int) -> list[dict]:
    """Get list of changed files with patches."""
    try:
        data = json.loads(gh_api(
            f"repos/{GITHUB_REPO}/pulls/{pr_number}/files",
            ["--paginate"],
        ))
        return data
    except Exception:
        return []


def read_full_source_tree() -> str:
    """
    Read the full Rust source tree from the checked-out repo.
    Gemini 3.1 Pro has a 1M token context — we can send the whole thing.
    """
    source_parts = []
    total_chars = 0
    max_chars = 500_000  # ~125k tokens, leaves plenty of room

    # Priority order: src/ first, then tests/, then other Rust files
    paths = []
    for pattern in ["src/**/*.rs", "tests/**/*.rs", "*.rs"]:
        for p in sorted(Path(".").glob(pattern)):
            if p not in paths:
                paths.append(p)

    for fpath in paths:
        try:
            content = fpath.read_text()
            if total_chars + len(content) > max_chars:
                source_parts.append(
                    f"\n### {fpath}\n(skipped — context budget reached)\n"
                )
                continue
            source_parts.append(f"\n### {fpath} ({len(content)} chars)\n```rust\n{content}\n```\n")
            total_chars += len(content)
        except (OSError, UnicodeDecodeError):
            pass

    return "".join(source_parts)


def read_project_files() -> dict[str, str]:
    """Read CLAUDE.md, Cargo.toml, and docs/ specs."""
    result = {}

    for fname in ["CLAUDE.md", "Cargo.toml"]:
        try:
            result[fname] = Path(fname).read_text()
        except (OSError, UnicodeDecodeError):
            pass

    docs_dir = Path("docs")
    if docs_dir.is_dir():
        for fpath in sorted(docs_dir.glob("*.md")):
            try:
                result[f"docs/{fpath.name}"] = fpath.read_text()
            except (OSError, UnicodeDecodeError):
                pass

    return result


def build_review_prompt(
    pr_info: dict,
    diff: str,
    changed_files: list[dict],
    comments: list[dict],
    linked_issue: dict | None,
    project_files: dict[str, str],
    source_tree: str,
) -> str:
    """Build the full-context review prompt for Gemini 3.1 Pro."""

    title = pr_info.get("title", "Unknown")
    body = pr_info.get("body", "") or ""
    author = pr_info.get("user", {}).get("login", "unknown")

    sections = []

    # Identity
    sections.append(
        "You are a senior code reviewer for the Copper Hollow project, "
        "a Rust application that composes MIDI music in folk/indie/alt-country "
        "styles. This is a production system sold to demanding customers. "
        "Your review is a critical quality gate."
    )

    # Project standards
    if "CLAUDE.md" in project_files:
        sections.append(f"## Project Coding Standards (CLAUDE.md)\n{project_files['CLAUDE.md']}")

    if "Cargo.toml" in project_files:
        sections.append(f"## Cargo.toml\n```toml\n{project_files['Cargo.toml']}\n```")

    # Specs
    spec_parts = []
    for name, content in project_files.items():
        if name.startswith("docs/"):
            spec_parts.append(f"### {name}\n{content}")
    if spec_parts:
        sections.append("## Specifications\n" + "\n\n".join(spec_parts))

    # Linked issue
    if linked_issue:
        sections.append(
            f"## Linked Issue #{linked_issue['number']}\n"
            f"**Title:** {linked_issue['title']}\n"
            f"**Labels:** {', '.join(linked_issue['labels'])}\n"
            f"**Description:**\n{linked_issue['body']}"
        )

    # PR details
    sections.append(
        f"## Pull Request #{pr_info.get('number', '?')}\n"
        f"**Title:** {title}\n"
        f"**Author:** {author}\n"
        f"**Description:**\n{body}"
    )

    # Existing comments/reviews
    if comments:
        comment_text = []
        for c in comments:
            prefix = f"[{c['type']}] {c['user']}"
            if c.get('path'):
                prefix += f" on {c['path']}"
            if c.get('line'):
                prefix += f":{c['line']}"
            if c.get('state'):
                prefix += f" ({c['state']})"
            comment_text.append(f"**{prefix}:**\n{c['body']}\n")
        sections.append(
            "## Existing Comments & Reviews\n" + "\n".join(comment_text)
        )

    # Full source tree
    sections.append(f"## Full Source Code\n{source_tree}")

    # Diff
    sections.append(f"## Pull Request Diff\n```diff\n{diff}\n```")

    # Changed files summary
    file_summary = []
    for f in changed_files:
        file_summary.append(
            f"- `{f.get('filename', '?')}` "
            f"({f.get('status', '?')}, "
            f"+{f.get('additions', 0)}/-{f.get('deletions', 0)})"
        )
    sections.append("## Changed Files\n" + "\n".join(file_summary))

    # Review instructions
    sections.append("""## Review Instructions

You have the FULL source code, the issue context, the PR description, existing
review comments, and the project specifications. Perform a thorough review:

1. **Issue compliance** — Does the implementation fully address the linked issue?
2. **Correctness** — Logic errors? Edge cases? Off-by-one errors?
3. **CLAUDE.md compliance** — No unwrap() in production, strong typing, engine/ purity (no IO/async/GUI), 480 ticks/beat, serde derives
4. **Spec compliance** — Does implementation match docs/ specifications?
5. **Production quality** — No stubs returning fake success, no mocks, no demoware, no silent failures
6. **Testing** — Adequate tests? Real behavior tested, not mocks? Edge cases covered?
7. **Security** — Panics, overflows, injection risks, unsafe patterns?
8. **Architecture** — Does the change fit the existing codebase patterns?
9. **Performance** — Any O(N²) or worse algorithms? Unnecessary allocations?

## Output Format

Respond with a JSON object:
```json
{
  "summary": "2-3 sentence overall assessment",
  "verdict": "APPROVE or REQUEST_CHANGES",
  "issue_compliance": "Does the PR fully address the linked issue? What's missing?",
  "findings": [
    {
      "severity": "high|medium|low",
      "file": "path/to/file.rs",
      "line": 42,
      "category": "correctness|compliance|quality|testing|security|performance",
      "message": "What's wrong and exactly how to fix it"
    }
  ]
}
```

Only output the JSON object, nothing else.
""")

    return "\n\n".join(sections)


def post_review(pr_number: int, review_data: dict) -> None:
    """Post review to the PR via GitHub API."""
    summary = review_data.get("summary", "Review complete.")
    verdict = review_data.get("verdict", "APPROVE")
    issue_compliance = review_data.get("issue_compliance", "")
    findings = review_data.get("findings", [])

    body_parts = [
        "## Gemini 3.1 Pro Code Review\n",
        f"**Verdict:** {verdict}\n",
        f"### Summary\n{summary}\n",
    ]

    if issue_compliance:
        body_parts.append(f"### Issue Compliance\n{issue_compliance}\n")

    if findings:
        # Group by severity
        high = [f for f in findings if f.get("severity") == "high"]
        medium = [f for f in findings if f.get("severity") == "medium"]
        low = [f for f in findings if f.get("severity") == "low"]

        for label, group in [("HIGH", high), ("MEDIUM", medium), ("LOW", low)]:
            if group:
                body_parts.append(f"### {label} Priority\n")
                for f in group:
                    file_path = f.get("file", "unknown")
                    line = f.get("line", "")
                    category = f.get("category", "")
                    message = f.get("message", "")
                    line_ref = f":{line}" if line else ""
                    cat_ref = f" [{category}]" if category else ""
                    body_parts.append(
                        f"- **`{file_path}{line_ref}`**{cat_ref}: {message}\n"
                    )

    body_parts.append(
        "\n---\n*Reviewed by Gemini 3.1 Pro with full source context "
        "(custom reviewer, not Gemini Code Assist)*"
    )

    body = "\n".join(body_parts)

    # Post as issue comment (always visible, doesn't conflict with other
    # review bots)
    try:
        gh_api(
            f"repos/{GITHUB_REPO}/issues/{pr_number}/comments",
            ["-f", f"body={body}"],
        )
        print("Posted issue comment.")
    except Exception as e:
        print(f"Warning: failed to post issue comment: {e}", file=sys.stderr)

    # Also post as a proper PR review
    event = "APPROVE" if verdict == "APPROVE" else "COMMENT"
    try:
        gh_api(
            f"repos/{GITHUB_REPO}/pulls/{pr_number}/reviews",
            ["-f", f"body={body}", "-f", f"event={event}"],
        )
        print("Posted PR review.")
    except Exception as e:
        print(f"Note: PR review post returned: {e}", file=sys.stderr)


def main():
    pr_number = int(os.environ.get("PR_NUMBER", "0"))
    api_key = os.environ.get("GEMINI_API_KEY", "")

    if not pr_number:
        print("ERROR: PR_NUMBER not set", file=sys.stderr)
        sys.exit(1)
    if not api_key:
        print("ERROR: GEMINI_API_KEY not set", file=sys.stderr)
        sys.exit(1)

    print(f"Reviewing PR #{pr_number} with {GEMINI_MODEL} (full context)...")

    # Gather ALL context
    print("Fetching PR info...")
    pr_info = get_pr_info(pr_number)
    print(f"PR: {pr_info.get('title', 'unknown')}")

    print("Fetching diff...")
    diff = get_pr_diff(pr_number)
    print(f"Diff: {len(diff)} chars")

    print("Fetching changed files...")
    changed_files = get_changed_files_list(pr_number)
    print(f"Changed files: {len(changed_files)}")

    print("Fetching comments and reviews...")
    comments = get_pr_comments(pr_number)
    print(f"Comments/reviews: {len(comments)}")

    print("Fetching linked issue...")
    linked_issue = get_linked_issue(pr_info)
    if linked_issue:
        print(f"Linked issue: #{linked_issue['number']} — {linked_issue['title']}")

    print("Reading project files (CLAUDE.md, Cargo.toml, docs/)...")
    project_files = read_project_files()
    print(f"Project files: {list(project_files.keys())}")

    print("Reading full source tree...")
    source_tree = read_full_source_tree()
    print(f"Source tree: {len(source_tree)} chars")

    # Build prompt
    prompt = build_review_prompt(
        pr_info, diff, changed_files, comments,
        linked_issue, project_files, source_tree,
    )
    print(f"Total prompt: {len(prompt)} chars (~{len(prompt)//4} tokens)")

    # Call Gemini 3.1 Pro
    print(f"Calling {GEMINI_MODEL}...")
    client = genai.Client(api_key=api_key)
    response = client.models.generate_content(
        model=GEMINI_MODEL,
        contents=[prompt],
        config=types.GenerateContentConfig(
            temperature=0.2,
            max_output_tokens=8192,
        ),
    )

    response_text = response.text.strip()
    print(f"Response: {len(response_text)} chars")

    # Parse JSON (handle markdown code blocks)
    cleaned = response_text
    if cleaned.startswith("```"):
        lines = cleaned.split("\n")
        lines = [l for l in lines if not l.strip().startswith("```")]
        cleaned = "\n".join(lines)

    try:
        review_data = json.loads(cleaned)
    except json.JSONDecodeError as e:
        print(f"Warning: JSON parse failed: {e}", file=sys.stderr)
        print(f"Raw response:\n{response_text[:500]}", file=sys.stderr)
        review_data = {
            "summary": response_text[:2000],
            "verdict": "COMMENT",
            "findings": [],
        }

    verdict = review_data.get("verdict", "unknown")
    findings = review_data.get("findings", [])
    high = sum(1 for f in findings if f.get("severity") == "high")
    print(f"Verdict: {verdict}")
    print(f"Findings: {len(findings)} total, {high} high priority")

    # Post to GitHub
    post_review(pr_number, review_data)
    print("Done.")


if __name__ == "__main__":
    main()
