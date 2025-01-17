<p align="center">
  <img src="assets/logo.svg" alt="Vouch" width="150" height="150" />
</p>

<h1 align="center">Vouch</h1>

<p align="center">🔍 A multi-ecosystem distributed package review system. 🔍</p>

<p align="center">
  <a href="https://matrix.to/#/#vouch:matrix.org"><img src="https://img.shields.io/matrix/vouch:matrix.org?label=chat&logo=matrix" alt="Matrix"></a>
</p>

## Introduction

Software packages are usually used without review. Who's checked the code? Typically, no one but the author. Vouch is a review system designed to solve this problem.

Vouch evaluates software dependencies using user generated micro-reviews. Even single line reviews become powerful when aggregated!

## Getting Started

### Setup

First, lets setup Vouch. During setup we can optionally specify a git repository URL for publishing reviews.

`vouch setup https://github.com/<username>/reviews`

### Extensions

Extensions enable Vouch to create reviews for packages from different ecosystems. For example, the [Python extension](https://github.com/vouch-dev/vouch-py) adds support for [pypi.org](https://pypi.org) packages. By default, Vouch includes extensions for Python and Javascript. Add an extension using the following command:

`vouch extension add py`

or via any GitHub repository URL:

`vouch extension add https://github.com/vouch-dev/vouch-py`

#### Official Extensions

| Name                                                        | Ecosystem      | Package Registries |
|-------------------------------------------------------------|----------------|--------------------|
| [vouch-py](https://github.com/vouch-dev/vouch-py)           | Python         | pypi.org           |
| [vouch-js](https://github.com/vouch-dev/vouch-js)           | Javascript     | npmjs.com          |
| [vouch-ansible](https://github.com/vouch-dev/vouch-ansible) | Ansible Galaxy | galaxy.ansible.com |

### Review

(Note: Vouch currently requires [VSCode](https://code.visualstudio.com/) to create reviews.)

Vouch supports multiple ecosystems and is extendable. For now, Python and Javascript support comes built-in. Lets review the [NPM](https://www.npmjs.com/) Javascript package [d3](https://www.npmjs.com/package/d3) at version `4.10.0`:

`vouch review d3 4.10.0`

### Peers

Subscribe to reviews created by other users using the command:

`vouch peer add https://github.com/vouch-dev/example-reviews`

### Sync

The sync command pulls new reviews from peers and publishes user generated reviews:

`vouch sync`

### Check

Reviews created using Vouch can be used to evaluate software project dependencies. Vouch extensions can discover ecosystem specific dependency definition files. For example, the Python extension parses `Pipfile.lock` files.

The `check` command generates an evaluation report of local project dependencies based on available reviews:

`vouch check`
