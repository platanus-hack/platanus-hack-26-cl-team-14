import { execFile } from "node:child_process";
import { promisify } from "node:util";

const run = promisify(execFile);
const repository = 'repository(owner: "platanus-hack", name: "platanus-hack-26-cl-team-14")';
const pageInfo = "pageInfo { hasNextPage endCursor }";
const timeline = `timelineItems(first: 20, since: $since, itemTypes: [ISSUE_COMMENT, CLOSED_EVENT], after: $timelineCursor) {
  nodes {
    __typename
    ... on IssueComment { id createdAt author { login } body url }
    ... on ClosedEvent { id createdAt actor { login } url }
  }
  ${pageInfo}
}`;
const issuesQuery = `query($since: DateTime!, $cursor: String, $timelineCursor: String) {
  ${repository} {
    issues(first: 20, after: $cursor, filterBy: {since: $since}, orderBy: {field: CREATED_AT, direction: ASC}) {
      nodes { id number title createdAt author { login } body url ${timeline} }
      ${pageInfo}
    }
  }
}`;
const timelineQuery = `query($number: Int!, $since: DateTime!, $timelineCursor: String!) {
  ${repository} { issue(number: $number) { ${timeline} } }
}`;

async function queryGitHub(query, variables) {
  const args = ["api", "graphql", "-f", `query=${query}`];
  for (const [key, value] of Object.entries(variables)) {
    if (value != null) args.push(typeof value === "number" ? "-F" : "-f", `${key}=${value}`);
  }
  const { stdout } = await run("gh", args, { maxBuffer: 128 * 1024 * 1024 });
  return JSON.parse(stdout).data.repository;
}

// The caller saves the time window and deduplicates overlapping windows by node ID.
export async function issueActivity(since, until) {
  const events = [];
  const start = Date.parse(since);
  const end = Date.parse(until);
  function collect(issue, node, kind) {
    const time = Date.parse(node.createdAt);
    if (time < start || time > end) return;
    events.push({
      key: node.id, at: new Date(time).toISOString(), number: issue.number, title: issue.title,
      kind, author: (node.author ?? node.actor)?.login ?? "ghost", body: node.body ?? "", url: node.url,
    });
  }

  let cursor;
  do {
    const { issues } = await queryGitHub(issuesQuery, { since, cursor });
    for (const issue of issues.nodes) {
      collect(issue, issue, "opened");
      let page = issue.timelineItems;
      for (;;) {
        for (const node of page.nodes) {
          collect(issue, node, node.__typename === "ClosedEvent" ? "closed" : "commented");
        }
        if (!page.pageInfo.hasNextPage) break;
        const next = await queryGitHub(timelineQuery, {
          since, number: issue.number, timelineCursor: page.pageInfo.endCursor,
        });
        page = next.issue.timelineItems;
      }
    }
    cursor = issues.pageInfo.hasNextPage ? issues.pageInfo.endCursor : null;
  } while (cursor);
  return events.sort((a, b) => a.at.localeCompare(b.at));
}
