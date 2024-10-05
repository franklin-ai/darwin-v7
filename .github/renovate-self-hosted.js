/**
 * Config for the self-hosted renovate app that is run via a github action
 *
 * See .github/workflows/renovate.yml
 */
module.exports = {
  autodiscover: false,
  extends: ['config:recommended'],
  platform: 'github',
  repositories: ["franklin-ai/darwin-v7"],
  prConcurrentLimit: 20,
  prCommitsPerRunLimit: 10,
};
