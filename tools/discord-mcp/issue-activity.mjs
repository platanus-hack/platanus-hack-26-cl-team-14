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
const issuesUrl = "https://github.com/platanus-hack/platanus-hack-26-cl-team-14/issues/";
const idsQuery = `query($ids: [ID!]!) {
  nodes(ids: $ids) {
    __typename
    ... on Issue { id number title createdAt author { login } body url }
    ... on IssueComment { id createdAt author { login } body url parent: issue { number title url } }
    ... on ClosedEvent { id createdAt actor { login } url parent: closable { ... on Issue { number title url } } }
  }
}`;

async function queryGitHub(query, variables) {
  const args = ["api", "graphql", "-f", `query=${query}`];
  for (const [key, value] of Object.entries(variables)) {
    if (Array.isArray(value)) {
      for (const id of value) args.push("-f", `${key}[]=${id}`);
    } else if (value != null) args.push(typeof value === "number" ? "-F" : "-f", `${key}=${value}`);
  }
  try {
    const { stdout } = await run("gh", args, { maxBuffer: 128 * 1024 * 1024 });
    return JSON.parse(stdout).data;
  } catch (error) {
    // GitHub can return the surviving nodes alongside a NOT_FOUND error for deleted IDs.
    if (query !== idsQuery || !error.stdout) throw error;
    const { data, errors } = JSON.parse(error.stdout);
    if (data?.nodes && errors?.length && errors.every(e => e.type === "NOT_FOUND" && e.path?.[0] === "nodes")) return data;
    throw error;
  }
}

function eventFromNode(issue, node, kind) {
  return {
    key: node.id, at: new Date(node.createdAt).toISOString(), number: issue.number, title: issue.title,
    kind, author: (node.author ?? node.actor)?.login ?? "ghost", body: node.body ?? "", url: node.url,
  };
}

export async function issueActivityById(keys) {
  const ids = [...new Set(keys)];
  const events = [];
  for (let i = 0; i < ids.length; i += 50) {
    const { nodes } = await queryGitHub(idsQuery, { ids: ids.slice(i, i + 50) });
    for (const node of nodes) {
      if (!node) continue;
      const issue = node.__typename === "Issue" ? node : node.parent;
      if (!issue?.url?.startsWith(issuesUrl)) continue;
      const kind = { Issue: "opened", IssueComment: "commented", ClosedEvent: "closed" }[node.__typename];
      events.push(eventFromNode(issue, node, kind));
    }
  }
  return events.sort((a, b) => a.at.localeCompare(b.at));
}

// The caller saves the time window and deduplicates overlapping windows by node ID.
export async function issueActivity(since, until) {
  const events = [];
  const start = Date.parse(since);
  const end = Date.parse(until);
  function collect(issue, node, kind) {
    const time = Date.parse(node.createdAt);
    if (time < start || time > end) return;
    events.push(eventFromNode(issue, node, kind));
  }

  let cursor;
  do {
    const { repository: { issues } } = await queryGitHub(issuesQuery, { since, cursor });
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
        page = next.repository.issue.timelineItems;
      }
    }
    cursor = issues.pageInfo.hasNextPage ? issues.pageInfo.endCursor : null;
  } while (cursor);
  return events.sort((a, b) => a.at.localeCompare(b.at));
}
