/**
 * Config for the self-hosted renovate app that is run via a github action
 *
 * See .github/workflows/renovate.yml
 */
module.exports = {
  platform: "github",
  repositories: ["franklin-ai/darwin-v7"],
};
