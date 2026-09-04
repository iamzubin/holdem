---
description: Resolve fixed bot review threads on the current PR (reply + resolveReviewThread)
agent: build
---

`/oc` runs already receive PR/issue thread history as prompt context. Use this when asked to fix review feedback and/or resolve threads.

1. Auth: `export GH_TOKEN="$GITHUB_TOKEN"` if `gh` reports auth errors. Derive `OWNER`/`REPO` from `$GITHUB_REPOSITORY` and `PR_NUMBER` from `$GITHUB_EVENT_PATH` (`jq -r .pull_request.number`) when running in CI; locally pass them explicitly.

2. List unresolved threads (thread node `id` starts with `PRRT_`):

```bash
gh api graphql -f query='query($o:String!,$r:String!,$n:Int!){repository(owner:$o,name:$r){pullRequest(number:$n){reviewThreads(first:100){nodes{id,isResolved,isOutdated,path,line,comments(first:20){nodes{body,author{login},createdAt}}}}}}}' -f o="$OWNER" -f r="$REPO" -F n="$PR_NUMBER"
```

3. Only resolve bot threads (`github-actions[bot]` / `opencode-agent[bot]`) where `isResolved==false`, `isOutdated==false`, and the fix is verified in the working tree (`read`/`grep` the file, check `gh pr diff`). Never resolve human threads, questions, or deferred items.

4. Reply first, then resolve:

```bash
gh api graphql -f query='mutation($id:ID!,$body:String!){addPullRequestReviewThreadReply(input:{pullRequestReviewThreadId:$id,body:$body}){comment{id}}}' -f id="PRRT_..." -f body="Fixed in <sha>: <what changed>"
gh api graphql -f query='mutation($id:ID!){resolveReviewThread(input:{threadId:$id}){thread{isResolved}}}' -f id="PRRT_..."
```

5. General PR comments are unresolvable — reply via a normal PR comment instead, never attempt GraphQL resolve on them.
