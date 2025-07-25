import { match } from "path-to-regexp";

import { type HuggingFaceModelReference } from "./schemas/HuggingFaceModelReference";

type UrlParams = {
  owner: string;
  repo: string;
  revision: string;
  filename: string;
};

const blobMatcher = match<UrlParams>("/:owner/:repo/blob/:revision/:filename");
const resolveMatcher = match<UrlParams>(
  "/:owner/:repo/resolve/:revision/:filename",
);

function urlParamsToHuggingFaceUrlParts({
  owner,
  repo,
  revision,
  filename,
}: UrlParams): HuggingFaceModelReference {
  return {
    filename: filename.startsWith("/") ? filename.slice(1) : filename,
    repo_id: `${owner}/${repo}`,
    revision,
  };
}

export function extractHuggingFaceUrlParts({
  pathname,
}: URL): HuggingFaceModelReference {
  const blobMatch = blobMatcher(pathname);

  if (blobMatch) {
    return urlParamsToHuggingFaceUrlParts(blobMatch.params);
  }

  const resolveMatch = resolveMatcher(pathname);

  if (resolveMatch) {
    return urlParamsToHuggingFaceUrlParts(resolveMatch.params);
  }

  throw new Error(`Invalid Hugging Face URL format: ${pathname}`);
}
