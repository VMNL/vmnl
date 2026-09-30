# IA Workflow for VMNL

## ChatGPT Plus

- Audit date: **2026-09-23**. Objective: preserve the included ChatGPT Plus allowance
  without purchasing credits while maintaining VMNL's engineering standards.

  This is a recommendation, not a benchmark result or a mandatory model policy. No
  comparative model runs were performed on VMNL, and the account's remaining allowance
  was not inspected. Product facts are a dated snapshot: recheck the official sources
  and the account dashboard before relying on availability, prices, or limits.

### Verified Product Facts

  The [API changelog](https://developers.openai.com/api/docs/changelog) dates GPT-6 Sol
  and Luna's release to September 22, 2026. The documented family contains Astra, Sol,
  and Luna; the consulted sources do not establish a GPT-6 Terra release. GPT-5.6 Terra
  is a separate, previous-generation model.

  The [Codex model guide](https://learn.chatgpt.com/docs/models) recommends Sol for
  complex coding and Luna for focused, repeatable work. Sol and Luna are offered through
  Plus in Codex and ChatGPT Work, subject to client availability and rollout, rather
  than ordinary Chat. The guide recommends starting with Sol Medium or Luna High.

  High means elevated reasoning; Extra High corresponds to `xhigh`. Max gives the
  selected model more time to reason. Ultra uses subagents; Luna supports Max but not
  Ultra. Effort levels are not equivalent across generations. More reasoning uses more
  tokens and time; it does not establish equivalence to a stronger model.

### Allowance and Cost

  The [Codex pricing page](https://learn.chatgpt.com/docs/pricing) publishes these estimates:

  | Model | Estimated local messages per five hours on Plus | Relative Standard credit price at identical token volumes and cache mix |
  | --- | ---: | ---: |
  | GPT-6 Luna | 350–3,000 | 1x |
  | GPT-6 Sol | 15–150 | 20x |
  | GPT-5.6 Sol | 10–100 | 40x |
  | GPT-6 Astra | 5–45 | 100x |

  Credit ratios are calculated from published input, cached-input, and output rates;
  each category has the same ratio for these four models. They are not ratios of tasks
  completed or guaranteed Plus allowance. Message estimates are not specific to Max or
  fixed limits. Context, reasoning, tools, caching, and task length affect consumption;
  weekly limits may apply. Work and Codex share usage.

  For this objective, keep Standard speed. [Fast mode](https://learn.chatgpt.com/docs/agent-configuration/speed)
  uses 2.5x the Standard credit rate for these GPT-6 models and consumes included limits
  faster. API billing is separate from Plus; API prices do not define subscription allowance.

  Inspect actual allowance and reset times in the [usage dashboard](https://chatgpt.com/codex/settings/usage)
  or with `/status` in Codex CLI.

### Recommended Routing for VMNL

  This matrix is engineering judgment to test on representative work. Luna and Sol mean
  GPT-6. Select by unresolved decisions, safety consequences, and validation strength,
  rather than file count alone.

  | Task | Brainstorm or audit | Execution |
  | --- | --- | --- |
  | Symbol inventory, factual documentation, mechanical edits | Luna Medium | Luna Medium |
  | Reproduced local bug with an established contract | Luna High | Luna High |
  | Unit or facade API tests with an independent expected result | Luna High | Luna High |
  | Bounded feature using existing abstractions | Sol Medium | Luna High after a precise plan; Sol Medium if decisions remain |
  | Public API, ownership, or crate boundaries | Sol High | Sol Medium; High when invariants remain difficult |
  | Unsafe code, FFI, Vulkan synchronization, or audio callbacks | Sol High; Extra High for an identified difficult question | Sol High |
  | Intermittent platform, GPU, or threading failure | Sol High for hypotheses and measurement design | Luna High for prescribed instrumentation; Sol High for interpretation |
  | Difficult isolated algorithm with a strong oracle | Compare Luna Max with Sol Medium or High | Use the configuration supported by measurements |

  Use **Luna High** for economical, bounded execution, **Sol Medium** when implementation
  still requires design decisions, and **Sol High** for substantial audits and architecture.
  Extra High and Max are targeted escalations, not blanket defaults.

  Luna Max is a plausible economical option for a specified problem requiring substantial
  deduction. The consulted sources do not establish that it replaces Sol for VMNL's
  architecture, FFI, or concurrency work. Reserve Astra for unusually difficult, blocked
  work when available and justified by remaining allowance. Keep GPT-5.6 Sol as a reference
  if a new model shows a concrete regression, rather than as the default by habit.

  Model selection does not relax [architecture contracts](architecture.md),
  [test evidence requirements](testing.md), or the validation order in
  [CONTRIBUTING.md](../CONTRIBUTING.md#before-submission). Confidence cannot replace native
  platform, GPU, ABI, or operator evidence. Required checks are independent of the model.

### Working Between Stages

  For a simple task, use one Luna High pass with an invariant, scope, and acceptance check.
  A separate brainstorm is useful when it resolves an actual decision. For structural
  work, use Sol to establish the contract before handing mechanical implementation to
  Luna. Keep Sol on execution when safety or design decisions remain.

  After an inconclusive attempt, follow repository rules: inspect the diff, restate the
  invariant, and obtain a targeted measurement before broadening the work. As a routing
  heuristic, escalate after two attempts without new evidence, or immediately when an
  invariant is misunderstood. Transfer the reproduction, diff, and observations instead
  of restarting the investigation.

  Use a focused Sol review for critical changes. Review observable contracts and evidence,
  not merely agreement with the earlier plan. A stronger reviewer cannot replace missing
  validation.

### Changing Models in One Conversation

  Codex supports model changes within an interactive session through its picker or
  `/model`. Keep related work together and change at stage boundaries instead of every
  message. Conversation continuity does not guarantee that every earlier detail remains
  in active context: compaction can shorten prior content. Do not assume another model
  receives or reproduces all of its predecessor's internal reasoning. Make useful
  conclusions explicit.

  Before handing execution to another model, record a concise handoff in the conversation:

  - accepted decisions and relevant rationale;
  - invariants and excluded scope;
  - affected files and current patch state;
  - required checks and evidence already obtained;
  - unresolved questions and blockers.

  A switch can reduce cache reuse and require additional context processing, with possible
  first-turn latency and usage effects. The [API caching guide](https://developers.openai.com/api/docs/guides/prompt-caching#which-settings-affect-the-cached-prefix)
  identifies model selection and effort as cache-affecting settings. This is API-level
  evidence, not a measured Codex Plus switching penalty. Exact allowance impact, cache
  reuse across model pairs, and client behavior were not verified. Even an unchanged
  session does not guarantee a cache hit.

  Prefer the same conversation for the same problem. A new conversation with a concise
  handoff may help when abandoned approaches dominate the history, but can omit useful
  context and lose cache reuse. Measure instead of assuming a new thread is cheaper.
  A plan from Sol does not give Luna Sol's ability to resolve remaining ambiguities.

### Minimal Evaluation Protocol

  To compare Luna High, Luna Max, and Sol Medium economically:

  1. Choose two small representative fixes with independently defined acceptance checks.
     Use the same starting commit, instructions, tools, and initial context for each
     configuration; isolate candidate solutions from one another.
  2. Record model, effort, client, speed, allowance before and after, elapsed time,
     correction count, and human review time. Avoid concurrent sessions and allowance
     reset boundaries. If dashboard precision cannot distinguish candidates, report the
     consumption comparison as inconclusive.
  3. Apply the same required checks. Reject contract violations or claims stronger than
     the actual evidence, even when tests pass.
  4. Compare allowance consumed through acceptance, including corrections and review,
     rather than initial response cost. Track review time separately. Two fixes are a
     pilot, not evidence across all VMNL tasks; repeat only when uncertainty justifies
     spending more quota.

  Working hypothesis: Luna High covers bounded implementation economically, while Sol
  pays for itself on unresolved contracts and difficult diagnosis. Whether Luna Max
  improves accepted-result cost over either remains to be measured.
