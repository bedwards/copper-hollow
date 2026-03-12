#!/usr/bin/env python3
"""
RALPH — Research, Arrange, Loop, Produce, Heal

An autonomous coding loop that orchestrates Claude Code CLI instances
to continuously improve the Copper Hollow project.

Each phase runs a single-shot Claude Code CLI instance with a focused
prompt and narrow scope. Status files in .ralph/ track state between
phases. Everything is serial, robust, and logged to stdout.

Usage:
    python3 ralph.py [--max-loops N] [--start-phase PHASE] [--dry-run]
"""

import argparse
import datetime
import json
import os
import subprocess
import sys
import time
import traceback

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

PROJECT_DIR = os.path.dirname(os.path.abspath(__file__))
RALPH_DIR = os.path.join(PROJECT_DIR, ".ralph")
STATUS_FILE = os.path.join(RALPH_DIR, "status.json")
METRICS_FILE = os.path.join(RALPH_DIR, "metrics.json")
BACKLOG_FILE = os.path.join(RALPH_DIR, "backlog.json")
RESEARCH_OUTPUT = os.path.join(RALPH_DIR, "research_output.json")
PROMPTS_DIR = os.path.join(RALPH_DIR, "prompts")
LOGS_DIR = os.path.join(RALPH_DIR, "logs")

PHASES = ["research", "plan", "orchestrate", "work", "review", "monitor"]

# Claude Code CLI settings
CLAUDE_CMD = "claude"
CLAUDE_MODEL = "opus"  # Always use Opus with max effort
CLAUDE_MAX_TURNS = 25  # Prevent runaway
CLAUDE_BUDGET = 5.0  # USD per invocation

# Error handling
MAX_CONSECUTIVE_ERRORS = 5
RATE_LIMIT_BACKOFF_SECONDS = 300  # 5 minutes
ERROR_BACKOFF_SECONDS = 30

# Gemini Code Assist review settings
GEMINI_WAIT_SECONDS = 180  # Wait 3 min (average is 1-4 min) before review phase
GEMINI_POLL_INTERVAL = 30  # Poll every 30s after initial wait
GEMINI_POLL_MAX_ATTEMPTS = 6  # Up to 3 more minutes of polling
GITHUB_REPO = "bedwards/copper-hollow"

# Tagging milestones (PR merge count -> tag)
MILESTONE_TAGS = {
    1: "v0.0.1",   # first PR merged (scaffolding)
    5: "v0.1.0",   # core types
    10: "v0.2.0",  # first engine modules
    20: "v0.3.0",  # CLI working
    30: "v0.4.0",  # GUI working
    50: "v0.5.0",  # full pipeline
}


def now_iso():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def log(msg, level="INFO"):
    ts = datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    print(f"[{ts}] [{level}] {msg}", flush=True)


def log_phase(phase, msg):
    log(f"[{phase.upper()}] {msg}")


# ---------------------------------------------------------------------------
# Status file helpers
# ---------------------------------------------------------------------------

def read_json(path):
    try:
        with open(path, "r") as f:
            return json.load(f)
    except (FileNotFoundError, json.JSONDecodeError):
        return {}


def write_json(path, data):
    with open(path, "w") as f:
        json.dump(data, f, indent=2, default=str)
        f.write("\n")


def update_status(**kwargs):
    status = read_json(STATUS_FILE)
    status.update(kwargs)
    status["updated_at"] = now_iso()
    write_json(STATUS_FILE, status)
    return status


def update_metrics(**kwargs):
    metrics = read_json(METRICS_FILE)
    for k, v in kwargs.items():
        if k == "phases_completed" and isinstance(v, dict):
            if "phases_completed" not in metrics:
                metrics["phases_completed"] = {}
            metrics["phases_completed"].update(v)
        else:
            metrics[k] = v
    write_json(METRICS_FILE, metrics)
    return metrics


def increment_metric(key, amount=1):
    metrics = read_json(METRICS_FILE)
    metrics[key] = metrics.get(key, 0) + amount
    write_json(METRICS_FILE, metrics)


def increment_phase_metric(phase):
    metrics = read_json(METRICS_FILE)
    if "phases_completed" not in metrics:
        metrics["phases_completed"] = {}
    metrics["phases_completed"][phase] = metrics["phases_completed"].get(phase, 0) + 1
    write_json(METRICS_FILE, metrics)


# ---------------------------------------------------------------------------
# Claude Code CLI runner
# ---------------------------------------------------------------------------

def run_claude(prompt, model=None, max_turns=None, budget=None,
               extra_flags=None, timeout=600):
    """
    Run a single-shot Claude Code CLI instance and return its output.
    All tools available, permissions skipped for full autonomy.
    Returns (success: bool, output: str, parsed_json: dict|None)
    """
    cmd = [CLAUDE_CMD, "-p", prompt]

    cmd.extend(["--model", model or CLAUDE_MODEL])
    cmd.extend(["--effort", "max"])
    cmd.extend(["--dangerously-skip-permissions"])
    cmd.extend(["--max-turns", str(max_turns or CLAUDE_MAX_TURNS)])

    if budget:
        cmd.extend(["--max-budget-usd", str(budget)])

    if extra_flags:
        cmd.extend(extra_flags)

    log(f"Running: claude -p '<prompt>' --model {model or CLAUDE_MODEL} "
        f"--max-turns {max_turns or CLAUDE_MAX_TURNS}")

    try:
        result = subprocess.run(
            cmd,
            cwd=PROJECT_DIR,
            capture_output=True,
            text=True,
            timeout=timeout,
        )

        stdout = result.stdout.strip()
        stderr = result.stderr.strip()

        if stderr:
            for line in stderr.split("\n"):
                log(f"  stderr: {line}", "WARN")

        # Check for rate limiting signals
        if result.returncode != 0:
            combined = (stdout + " " + stderr).lower()
            if any(x in combined for x in [
                "rate limit", "429", "overloaded", "too many requests",
                "quota exceeded", "capacity"
            ]):
                log("Rate limit detected!", "WARN")
                update_metrics(last_rate_limit_at=now_iso())
                return False, stdout, None

        if result.returncode != 0:
            log(f"Claude exited with code {result.returncode}", "ERROR")
            return False, stdout, None

        # Try to parse JSON from output
        parsed = try_parse_json(stdout)

        return True, stdout, parsed

    except subprocess.TimeoutExpired:
        log(f"Claude timed out after {timeout}s", "ERROR")
        return False, "", None
    except FileNotFoundError:
        log("Claude CLI not found. Is it installed and on PATH?", "FATAL")
        sys.exit(1)


def try_parse_json(text):
    """Extract JSON from claude output, handling markdown fences."""
    # Try direct parse
    try:
        return json.loads(text)
    except (json.JSONDecodeError, ValueError):
        pass

    # Try to find JSON in markdown code blocks
    import re
    patterns = [
        r"```json\s*\n(.*?)\n\s*```",
        r"```\s*\n(\{.*?\})\n\s*```",
        r"(\{[^{}]*(?:\{[^{}]*\}[^{}]*)*\})",
    ]
    for pattern in patterns:
        match = re.search(pattern, text, re.DOTALL)
        if match:
            try:
                return json.loads(match.group(1))
            except (json.JSONDecodeError, ValueError):
                continue

    return None


# ---------------------------------------------------------------------------
# Prompt builders
# ---------------------------------------------------------------------------

def load_prompt_template(name):
    path = os.path.join(PROMPTS_DIR, f"{name}.md")
    with open(path, "r") as f:
        return f.read()


def build_research_prompt():
    template = load_prompt_template("researcher")
    status = read_json(STATUS_FILE)
    metrics = read_json(METRICS_FILE)
    return (
        f"{template}\n\n"
        f"## Current State\n"
        f"- Loop count: {metrics.get('total_loops', 0)}\n"
        f"- PRs merged so far: {metrics.get('total_prs_merged', 0)}\n"
        f"- Last phase: {status.get('last_phase_completed', 'none')}\n"
    )


def build_plan_prompt():
    template = load_prompt_template("planner")
    return template


def build_orchestrate_prompt():
    template = load_prompt_template("orchestrator")
    metrics = read_json(METRICS_FILE)
    return (
        f"{template}\n\n"
        f"## Metrics\n"
        f"- Total PRs merged: {metrics.get('total_prs_merged', 0)}\n"
        f"- Total issues created: {metrics.get('total_issues_created', 0)}\n"
    )


def build_work_prompt(issue_number, issue_title, branch_name):
    template = load_prompt_template("worker")
    return (
        template
        .replace("{issue_number}", str(issue_number))
        .replace("{issue_title}", issue_title)
        .replace("{branch_name}", branch_name)
    )


def build_review_prompt(pr_number, branch_name):
    template = load_prompt_template("reviewer")
    return (
        template
        .replace("{pr_number}", str(pr_number))
        .replace("{branch_name}", branch_name)
    )


def build_monitor_prompt():
    return load_prompt_template("monitor")


# ---------------------------------------------------------------------------
# Git helpers
# ---------------------------------------------------------------------------

def git_run(*args, check=True):
    """Run a git command and return stdout."""
    result = subprocess.run(
        ["git"] + list(args),
        cwd=PROJECT_DIR,
        capture_output=True,
        text=True,
    )
    if check and result.returncode != 0:
        log(f"git {' '.join(args)} failed: {result.stderr}", "ERROR")
    return result.stdout.strip(), result.returncode


def wait_for_gemini_review(pr_number):
    """Wait for Gemini Code Assist to post its review on a PR.
    Returns True if Gemini review found, False if timed out."""
    log_phase("review", f"Waiting {GEMINI_WAIT_SECONDS}s for Gemini Code Assist to review PR #{pr_number}...")
    time.sleep(GEMINI_WAIT_SECONDS)

    for attempt in range(1, GEMINI_POLL_MAX_ATTEMPTS + 1):
        try:
            result = subprocess.run(
                ["gh", "api", f"repos/{GITHUB_REPO}/pulls/{pr_number}/reviews",
                 "--jq", '[.[] | select(.user.login == "gemini-code-assist[bot]")] | length'],
                cwd=PROJECT_DIR, capture_output=True, text=True, timeout=30,
            )
            count = int(result.stdout.strip() or "0")
            if count > 0:
                log_phase("review", f"Gemini review found after {GEMINI_WAIT_SECONDS + attempt * GEMINI_POLL_INTERVAL}s")
                return True
        except (subprocess.TimeoutExpired, ValueError):
            pass

        log_phase("review", f"No Gemini review yet, polling... ({attempt}/{GEMINI_POLL_MAX_ATTEMPTS})")
        time.sleep(GEMINI_POLL_INTERVAL)

    log_phase("review", "Gemini review not found within timeout — proceeding without it")
    return False


def verify_review_posted(pr_number):
    """Verify that a non-bot review was actually posted on the PR.
    Returns True if a human/RALPH review exists, False otherwise."""
    try:
        result = subprocess.run(
            ["gh", "api", f"repos/{GITHUB_REPO}/pulls/{pr_number}/reviews",
             "--jq", '[.[] | select(.user.login != "gemini-code-assist[bot]" and .user.login != "chatgpt-codex-connector[bot]")] | length'],
            cwd=PROJECT_DIR, capture_output=True, text=True, timeout=30,
        )
        count = int(result.stdout.strip() or "0")
        return count > 0
    except (subprocess.TimeoutExpired, ValueError):
        return False


def ensure_main():
    """Switch to main branch and pull latest."""
    git_run("checkout", "main")
    git_run("pull", "origin", "main")
    log("On main branch, pulled latest")


def create_branch(name):
    """Create and switch to a new branch from main."""
    ensure_main()
    stdout, rc = git_run("checkout", "-b", name)
    if rc != 0:
        # Branch might exist, try switching
        git_run("checkout", name)
    log(f"On branch: {name}")


def branch_name_for_issue(issue_number, issue_title):
    """Generate a branch name from issue number and title."""
    slug = issue_title.lower()
    slug = "".join(c if c.isalnum() or c == " " else "" for c in slug)
    slug = "-".join(slug.split()[:5])
    return f"issue-{issue_number}-{slug}"


# ---------------------------------------------------------------------------
# Phase implementations
# ---------------------------------------------------------------------------

def phase_research(dry_run=False):
    """Phase 1: Research the web and codebase for improvements."""
    log_phase("research", "Starting research phase")
    update_status(phase="research")

    prompt = build_research_prompt()

    if dry_run:
        log_phase("research", f"[DRY RUN] Would run researcher with {len(prompt)} char prompt")
        return True

    success, output, parsed = run_claude(
        prompt,
        model="opus",
        max_turns=15,
        timeout=300,
    )

    if success and parsed:
        write_json(RESEARCH_OUTPUT, parsed)
        findings_count = len(parsed.get("findings", []))
        log_phase("research", f"Found {findings_count} findings")
    elif success:
        # Claude returned text but not parseable JSON — save raw
        write_json(RESEARCH_OUTPUT, {"raw_output": output, "timestamp": now_iso()})
        log_phase("research", "Got output but couldn't parse JSON, saved raw")
    else:
        log_phase("research", "Research failed")
        return False

    increment_phase_metric("research")
    update_status(last_phase_completed="research")
    return True


def phase_plan(dry_run=False):
    """Phase 2: Create GitHub issues from research findings."""
    log_phase("plan", "Starting planning phase")
    update_status(phase="plan")

    prompt = build_plan_prompt()

    if dry_run:
        log_phase("plan", f"[DRY RUN] Would run planner with {len(prompt)} char prompt")
        return True

    success, output, parsed = run_claude(
        prompt,
        model="opus",
        max_turns=30,  # May need many turns to create multiple issues
        timeout=600,
    )

    if success and parsed:
        issues_created = parsed.get("issues_created", [])
        log_phase("plan", f"Created {len(issues_created)} issues")
        increment_metric("total_issues_created", len(issues_created))

        # Update backlog
        backlog = read_json(BACKLOG_FILE)
        for issue in issues_created:
            backlog.setdefault("issues", []).append(issue)
        backlog["last_groomed_at"] = now_iso()
        write_json(BACKLOG_FILE, backlog)
    elif success:
        log_phase("plan", "Planner finished but no structured output")
    else:
        log_phase("plan", "Planning failed")
        return False

    increment_phase_metric("plan")
    update_status(last_phase_completed="plan")
    return True


def phase_orchestrate(dry_run=False):
    """Phase 3: Groom backlog and select next issue."""
    log_phase("orchestrate", "Starting orchestration phase")
    update_status(phase="orchestrate")

    prompt = build_orchestrate_prompt()

    if dry_run:
        log_phase("orchestrate", f"[DRY RUN] Would run orchestrator")
        return True

    success, output, parsed = run_claude(
        prompt,
        model="opus",
        max_turns=10,
        timeout=180,
    )

    if success and parsed:
        selected = parsed.get("selected_issue")
        if selected:
            issue_num = selected["number"]
            issue_title = selected["title"]
            log_phase("orchestrate", f"Selected issue #{issue_num}: {issue_title}")
            update_status(
                current_issue=selected,
                current_branch=branch_name_for_issue(issue_num, issue_title),
            )

            # Update backlog with priority order
            backlog = read_json(BACKLOG_FILE)
            backlog["priority_order"] = parsed.get("priority_order", [])
            backlog["last_groomed_at"] = now_iso()
            write_json(BACKLOG_FILE, backlog)
        else:
            log_phase("orchestrate", "No issues ready to work on")
            update_status(current_issue=None, current_branch=None)
    else:
        log_phase("orchestrate", "Orchestration failed")
        return False

    increment_phase_metric("orchestrate")
    update_status(last_phase_completed="orchestrate")
    return True


def phase_work(dry_run=False):
    """Phase 4: Implement the selected issue on a feature branch."""
    log_phase("work", "Starting work phase")
    update_status(phase="work")

    status = read_json(STATUS_FILE)
    issue = status.get("current_issue")
    branch = status.get("current_branch")

    if not issue:
        log_phase("work", "No issue selected, skipping work phase")
        return True

    issue_number = issue["number"]
    issue_title = issue["title"]

    log_phase("work", f"Working on issue #{issue_number}: {issue_title}")
    log_phase("work", f"Branch: {branch}")

    # Create the feature branch
    create_branch(branch)

    prompt = build_work_prompt(issue_number, issue_title, branch)

    if dry_run:
        log_phase("work", f"[DRY RUN] Would implement issue #{issue_number}")
        ensure_main()
        return True

    success, output, parsed = run_claude(
        prompt,
        model="opus",
        max_turns=CLAUDE_MAX_TURNS,
        budget=CLAUDE_BUDGET,
        timeout=900,  # 15 minutes for implementation
    )

    if success and parsed:
        pr_number = parsed.get("pr_number")
        if pr_number:
            log_phase("work", f"Created PR #{pr_number}")
            update_status(current_pr=pr_number)
            increment_metric("total_prs_created")
        else:
            log_phase("work", "Work completed but no PR created")
    elif success:
        log_phase("work", "Worker finished but no structured output")
        # Try to detect PR from git state
        result = subprocess.run(
            ["gh", "pr", "list", "--head", branch, "--json", "number"],
            cwd=PROJECT_DIR, capture_output=True, text=True,
        )
        try:
            prs = json.loads(result.stdout)
            if prs:
                update_status(current_pr=prs[0]["number"])
                log_phase("work", f"Found PR #{prs[0]['number']} from branch")
        except (json.JSONDecodeError, IndexError, KeyError):
            pass
    else:
        log_phase("work", "Work phase failed")
        ensure_main()
        return False

    # Switch back to main
    ensure_main()

    increment_phase_metric("work")
    update_status(last_phase_completed="work")
    return True


def phase_review(dry_run=False):
    """Phase 5: Wait for Gemini, review the PR, enforce review was posted."""
    log_phase("review", "Starting review phase")
    update_status(phase="review")

    status = read_json(STATUS_FILE)
    pr_number = status.get("current_pr")
    branch = status.get("current_branch")

    if not pr_number:
        log_phase("review", "No PR to review, skipping")
        return True

    log_phase("review", f"Reviewing PR #{pr_number}")

    if dry_run:
        log_phase("review", f"[DRY RUN] Would review PR #{pr_number}")
        return True

    # Step 1: Wait for Gemini Code Assist to post its review
    gemini_found = wait_for_gemini_review(pr_number)

    # Step 2: Run the reviewer worker with enough turns and time
    prompt = build_review_prompt(str(pr_number), branch or "unknown")

    # Tell the reviewer whether Gemini posted
    if gemini_found:
        prompt += "\n\n## Gemini Status\nGemini Code Assist HAS posted its review. Read it before proceeding.\n"
    else:
        prompt += "\n\n## Gemini Status\nGemini Code Assist did NOT post a review within the timeout. Proceed with your own review but note this in the output.\n"

    success, output, parsed = run_claude(
        prompt,
        model="opus",
        max_turns=25,
        timeout=600,
    )

    if success and parsed:
        action = parsed.get("action", "unknown")
        log_phase("review", f"Review action: {action}")

        if action == "merged":
            # ENFORCEMENT: Verify a review was actually posted before accepting
            if not verify_review_posted(pr_number):
                log_phase("review",
                    "ENFORCEMENT FAILURE: Reviewer claims merged but no review "
                    "was posted on the PR. Rejecting result — will retry next loop.")
                increment_metric("total_errors")
                # Don't clear state — retry next loop
                increment_phase_metric("review")
                update_status(last_phase_completed="review")
                return True  # Don't fail the loop, just retry

            increment_metric("total_prs_merged")
            tag = parsed.get("tag_created")
            if tag:
                log_phase("review", f"Tagged: {tag}")

            # Clear current work state
            update_status(current_issue=None, current_branch=None, current_pr=None)

        elif action == "changes_requested":
            # Verify change request was actually posted
            if not verify_review_posted(pr_number):
                log_phase("review",
                    "ENFORCEMENT FAILURE: Reviewer claims changes requested but "
                    "no review was posted. Will retry next loop.")
                increment_phase_metric("review")
                update_status(last_phase_completed="review")
                return True

            notes = parsed.get("review_notes", "see PR comments")
            log_phase("review", f"Changes requested: {notes}")
            # Leave PR open for next work phase to address

        elif action == "waiting":
            log_phase("review", "Checks still running, will retry next loop")
    else:
        log_phase("review", "Review phase completed without structured output")

    increment_phase_metric("review")
    update_status(last_phase_completed="review")
    return True


def phase_monitor(dry_run=False):
    """Phase 6: Check main branch health."""
    log_phase("monitor", "Starting monitoring phase")
    update_status(phase="monitor")

    ensure_main()

    prompt = build_monitor_prompt()

    if dry_run:
        log_phase("monitor", "[DRY RUN] Would run health checks")
        return True

    success, output, parsed = run_claude(
        prompt,
        model="opus",
        max_turns=10,
        timeout=300,
    )

    if success and parsed:
        health = parsed.get("health", {})
        compiles = health.get("compiles", "unknown")
        tests = health.get("tests_pass", "unknown")
        clippy = health.get("clippy_clean", "unknown")

        log_phase("monitor", f"Health — compiles: {compiles}, tests: {tests}, clippy: {clippy}")

        warnings = parsed.get("warnings", [])
        for w in warnings:
            log_phase("monitor", f"Warning: {w}")
    else:
        log_phase("monitor", "Monitor completed without structured output")

    increment_phase_metric("monitor")
    update_status(last_phase_completed="monitor")
    return True


# ---------------------------------------------------------------------------
# Main loop
# ---------------------------------------------------------------------------

PHASE_FUNCTIONS = {
    "research": phase_research,
    "plan": phase_plan,
    "orchestrate": phase_orchestrate,
    "work": phase_work,
    "review": phase_review,
    "monitor": phase_monitor,
}


def check_should_halt():
    """Check if we should stop the loop."""
    status = read_json(STATUS_FILE)
    if status.get("halted"):
        log(f"Halted: {status.get('halt_reason', 'unknown')}", "WARN")
        return True

    metrics = read_json(METRICS_FILE)

    # Check backoff
    backoff_until = metrics.get("backoff_until")
    if backoff_until:
        try:
            until = datetime.datetime.fromisoformat(backoff_until)
            if datetime.datetime.now(datetime.timezone.utc) < until:
                remaining = (until - datetime.datetime.now(datetime.timezone.utc)).seconds
                log(f"In backoff period, {remaining}s remaining", "WARN")
                time.sleep(min(remaining, 60))  # Sleep in chunks
                return True
        except (ValueError, TypeError):
            pass

    # Check consecutive errors
    if metrics.get("consecutive_errors", 0) >= MAX_CONSECUTIVE_ERRORS:
        log(f"Too many consecutive errors ({MAX_CONSECUTIVE_ERRORS}), halting", "ERROR")
        update_status(halted=True, halt_reason="max consecutive errors reached")
        return True

    return False


def run_phase(phase_name, dry_run=False):
    """Run a single phase with error handling."""
    phase_fn = PHASE_FUNCTIONS.get(phase_name)
    if not phase_fn:
        log(f"Unknown phase: {phase_name}", "ERROR")
        return False

    try:
        success = phase_fn(dry_run=dry_run)
        if success:
            update_metrics(consecutive_errors=0)
        else:
            increment_metric("consecutive_errors")
            increment_metric("total_errors")
        return success

    except KeyboardInterrupt:
        raise
    except Exception as e:
        log(f"Phase {phase_name} crashed: {e}", "ERROR")
        traceback.print_exc()
        increment_metric("consecutive_errors")
        increment_metric("total_errors")
        update_status(last_error=str(e))
        return False


def main_loop(max_loops=None, start_phase=None, dry_run=False):
    """The main RALPH loop."""
    log("=" * 60)
    log("RALPH — Research, Arrange, Loop, Produce, Heal")
    log("=" * 60)
    log(f"Project: {PROJECT_DIR}")
    log(f"Max loops: {max_loops or 'unlimited'}")
    log(f"Start phase: {start_phase or PHASES[0]}")
    log(f"Dry run: {dry_run}")
    log("")

    # Determine starting phase index
    if start_phase:
        if start_phase not in PHASES:
            log(f"Invalid start phase: {start_phase}. Valid: {PHASES}", "ERROR")
            sys.exit(1)
        phase_idx = PHASES.index(start_phase)
    else:
        phase_idx = 0

    loop_count = 0

    while True:
        if max_loops and loop_count >= max_loops:
            log(f"Reached max loops ({max_loops}), exiting")
            break

        # Check halt conditions
        if check_should_halt():
            break

        loop_count += 1
        increment_metric("total_loops")

        log("")
        log(f"{'=' * 60}")
        log(f"LOOP {loop_count} starting at phase: {PHASES[phase_idx]}")
        log(f"{'=' * 60}")
        log("")

        update_status(
            loop_count=loop_count,
            started_at=now_iso() if phase_idx == 0 else None,
        )

        # Run phases in order starting from phase_idx
        for i in range(phase_idx, len(PHASES)):
            phase_name = PHASES[i]

            if check_should_halt():
                break

            success = run_phase(phase_name, dry_run=dry_run)

            if not success:
                # Check if it's a rate limit
                metrics = read_json(METRICS_FILE)
                if metrics.get("last_rate_limit_at"):
                    backoff = datetime.datetime.now(datetime.timezone.utc) + \
                        datetime.timedelta(seconds=RATE_LIMIT_BACKOFF_SECONDS)
                    update_metrics(
                        backoff_until=backoff.isoformat(),
                        last_rate_limit_at=None,
                    )
                    log(f"Rate limited — backing off for {RATE_LIMIT_BACKOFF_SECONDS}s", "WARN")
                    break

                # Non-rate-limit error: short backoff and continue
                log(f"Phase {phase_name} failed, waiting {ERROR_BACKOFF_SECONDS}s", "WARN")
                time.sleep(ERROR_BACKOFF_SECONDS)
                # Skip to next phase rather than halting entirely
                continue

        # Reset to start of phases for next loop
        phase_idx = 0

        # Brief pause between loops
        log("Loop complete, pausing 10s before next loop...")
        time.sleep(10)

    log("")
    log("=" * 60)
    log("RALPH loop ended")
    log(f"Total loops completed: {loop_count}")
    metrics = read_json(METRICS_FILE)
    log(f"Total issues created: {metrics.get('total_issues_created', 0)}")
    log(f"Total PRs created: {metrics.get('total_prs_created', 0)}")
    log(f"Total PRs merged: {metrics.get('total_prs_merged', 0)}")
    log(f"Total errors: {metrics.get('total_errors', 0)}")
    log("=" * 60)


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(
        description="RALPH — Autonomous coding loop for Copper Hollow"
    )
    parser.add_argument(
        "--max-loops", type=int, default=None,
        help="Maximum number of loops (default: unlimited)"
    )
    parser.add_argument(
        "--start-phase", choices=PHASES, default=None,
        help="Phase to start from (default: research)"
    )
    parser.add_argument(
        "--dry-run", action="store_true",
        help="Show what would be done without running Claude"
    )
    parser.add_argument(
        "--reset", action="store_true",
        help="Reset status and metrics files before starting"
    )
    parser.add_argument(
        "--status", action="store_true",
        help="Print current status and exit"
    )

    args = parser.parse_args()

    if args.status:
        status = read_json(STATUS_FILE)
        metrics = read_json(METRICS_FILE)
        print(json.dumps({"status": status, "metrics": metrics}, indent=2))
        return

    if args.reset:
        log("Resetting status and metrics files")
        write_json(STATUS_FILE, {
            "phase": "idle", "loop_count": 0,
            "last_phase_completed": None, "last_error": None,
            "started_at": None, "updated_at": now_iso(),
            "current_issue": None, "current_branch": None,
            "current_pr": None, "halted": False, "halt_reason": None,
        })
        write_json(METRICS_FILE, {
            "total_loops": 0, "total_issues_created": 0,
            "total_prs_created": 0, "total_prs_merged": 0,
            "total_errors": 0, "consecutive_errors": 0,
            "last_rate_limit_at": None, "backoff_until": None,
            "phases_completed": {p: 0 for p in PHASES},
        })

    try:
        main_loop(
            max_loops=args.max_loops,
            start_phase=args.start_phase,
            dry_run=args.dry_run,
        )
    except KeyboardInterrupt:
        log("\nInterrupted by user (Ctrl+C)", "WARN")
        update_status(phase="interrupted", halted=True, halt_reason="user interrupt")
        sys.exit(130)


if __name__ == "__main__":
    main()
