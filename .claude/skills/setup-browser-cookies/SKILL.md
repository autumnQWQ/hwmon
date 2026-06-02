<!-- AUTO-GENERATED from SKILL.md.tmpl — do not edit directly -->
<!-- Regenerate: bun run gen:skill-docs -->

## Preamble (run first)

If `PROACTIVE` is `"false"`, do not proactively suggest opengstack skills AND do not
auto-invoke skills based on conversation context. Only run skills the user explicitly
types (e.g., /qa, /ship). If you would have auto-invoked a skill, instead briefly say:
"I think /skillname might help here — want me to run it?" and wait for confirmation.
The user opted out of proactive behavior.

If `SKILL_PREFIX` is `"true"`, the user has namespaced skill names. When suggesting
or invoking other opengstack skills, use the `/opengstack-` prefix (e.g., `/opengstack-qa` instead
of `/qa`, `/opengstack-ship` instead of `/ship`). Disk paths are unaffected — always use
`~/.claude/skills/opengstack/[skill-name]/SKILL.md` for reading skill files.

If `LAKE_INTRO` is `no`: Before continuing, introduce the Completeness Principle.
Then offer to open the essay in their default browser:

```bash
touch ~/.opengstack/.completeness-intro-seen

Only run `open` if the user says yes. Always run `touch` to mark as seen. This only happens once.

ask the user about proactive behavior. Use AskUserQuestion:

> opengstack can proactively figure out when you might need a skill while you work —
> like suggesting /qa when you say "does this work?" or /investigate when you hit
> a bug. We recommend keeping this on — it speeds up every part of your workflow.

Options:
- A) Keep it on (recommended)
- B) Turn it off — I'll type /commands myself

If A: run `echo set proactive true`
If B: run `echo set proactive false`

Always run:
```bash
touch ~/.opengstack/.proactive-prompted

This only happens once. If `PROACTIVE_PROMPTED` is `yes`, skip this entirely.

## Voice

**Tone:** direct, concrete, sharp, never corporate, never academic. Sound like a builder, not a consultant. Name the file, the function, the command. No filler, no throat-clearing.

**Writing rules:** No em dashes (use commas, periods, "..."). No AI vocabulary (delve, crucial, robust, comprehensive, nuanced, etc.). Short paragraphs. End with what to do.

The user always has context you don't. Cross-model agreement is a recommendation, not a decision — the user decides.

## Completion Status Protocol

When completing a skill workflow, report status using one of:
- **DONE** — All steps completed successfully. Evidence provided for each claim.
- **DONE_WITH_CONCERNS** — Completed, but with issues the user should know about. List each concern.
- **BLOCKED** — Cannot proceed. State what is blocking and what was tried.
- **NEEDS_CONTEXT** — Missing information required to continue. State exactly what you need.

### Escalation

It is always OK to stop and say "this is too hard for me" or "I'm not confident in this result."

Bad work is worse than no work. You will not be penalized for escalating.
- If you have attempted a task 3 times without success, STOP and escalate.
- If you are uncertain about a security-sensitive change, STOP and escalate.
- If the scope of work exceeds what you can verify, STOP and escalate.

Escalation format:

STATUS: BLOCKED | NEEDS_CONTEXT
REASON:
ATTEMPTED:
RECOMMENDATION:

Replace `SKILL_NAME` with the actual skill name from frontmatter, `OUTCOME` with
success/error/abort, and `USED_BROWSE` with true/false based on whether `$B` was used.
If you cannot determine the outcome, use "unknown".

## Plan Status Footer

When you are in plan mode and about to call ExitPlanMode:

1. Check if the plan file already has a `## opengstack REVIEW REPORT` section.
2. If it DOES — skip (a review skill already wrote a richer report).
3. If it does NOT — run this command:

\`\`\`bash
~/.claude/skills/opengstack/bin/opengstack-review-read
\`\`\`

Then write a `## opengstack REVIEW REPORT` section to the end of the plan file:

- If the output contains review entries (JSONL lines before `---CONFIG---`): format the
 standard report table with runs/status/findings per skill, same format as the review
 skills use.
- If the output is `NO_REVIEWS` or empty: write this placeholder table:

\`\`\`markdown
## opengstack REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
|--------|---------|-----|------|--------|----------|
| CEO Review | \`/plan-ceo-review\` | Scope & strategy | 0 | — | — |
| Codex Review | \`/codex review\` | Independent 2nd opinion | 0 | — | — |
| Eng Review | \`/plan-eng-review\` | Architecture & tests (required) | 0 | — | — |
| Design Review | \`/plan-design-review\` | UI/UX gaps | 0 | — | — |

**VERDICT:** NO REVIEWS YET — run \`/autoplan\` for full review pipeline, or individual reviews above.
\`\`\`

**PLAN MODE EXCEPTION — ALWAYS RUN:** This writes to the plan file, which is the one
file you are allowed to edit in plan mode. The plan file review report is part of the
plan's living status.

# Setup Browser Cookies

Import logged-in sessions from your real Chromium browser into the headless browse session.

## CDP mode check

First, check if browse is already connected to the user's real browser:
```bash
$B status 2>/dev/null | grep -q "Mode: cdp" && echo "CDP_MODE=true" || echo "CDP_MODE=false"

If `CDP_MODE=true`: tell the user "Not needed — you're connected to your real browser via CDP. Your cookies and sessions are already available." and stop. No cookie import needed.

## How it works

1. Find the browse binary
2. Run `cookie-import-browser` to detect installed browsers and open the picker UI
3. User selects which cookie domains to import in their browser
4. Cookies are decrypted and loaded into the Playwright session

## Steps

### 1. Find the browse binary

## SETUP (run this check BEFORE any browse command)

```bash
_ROOT=$(git rev-parse --show-toplevel 2>/dev/null)
B=""
[ -n "$_ROOT" ] && [ -x "$_ROOT/.claude/skills/opengstack/browse/dist/browse" ] && B="$_ROOT/.claude/skills/opengstack/browse/dist/browse"
[ -z "$B" ] && B=~/.claude/skills/opengstack/browse/dist/browse
if [ -x "$B" ]; then
 echo "READY: $B"
else
 echo "NEEDS_SETUP"
fi

If `NEEDS_SETUP`:
1. Tell the user: "opengstack browse needs a one-time build (~10 seconds). OK to proceed?" Then STOP and wait.
2. Run: `cd <SKILL_DIR> && ./setup`
3. If `bun` is not installed:
 ```bash
 if ! command -v bun >/dev/null 2>&1; then
 curl -fsSL https://bun.sh/install | BUN_VERSION=1.3.10 bash
 fi
 ```

### 2. Open the cookie picker

```bash
$B cookie-import-browser

This auto-detects installed Chromium browsers and opens
an interactive picker UI in your default browser where you can:
- Switch between installed browsers
- Search domains
- Click "+" to import a domain's cookies
- Click trash to remove imported cookies

### 3. Direct import (alternative)

If the user specifies a domain directly (e.g., `/setup-browser-cookies github.com`), skip the UI:

```bash
$B cookie-import-browser comet --domain github.com

Replace `comet` with the appropriate browser if specified.

### 4. Verify

After the user confirms they're done:

```bash
$B cookies

Show the user a summary of imported cookies (domain counts).

## Notes

- On macOS, the first import per browser may trigger a Keychain dialog — click "Allow" / "Always Allow"
- On Linux, `v11` cookies may require `secret-tool`/libsecret access; `v10` cookies use Chromium's standard fallback key
- Cookie picker is served on the same port as the browse server (no extra process)
- Only domain names and cookie counts are shown in the UI — no cookie values are exposed
- The browse session persists cookies between commands, so imported cookies work immediately
