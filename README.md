# neomake

`neomake` is a fully open source task runner CLI utility that acts as a modern alternative to known utilities.

## Project state

`neomake` is currently a pre-release and to be considered as **unstable**. It is actively maintained and used in production.

## Features

- **Task chains**\
  Execute many tasks in sequence, easily re-use single tasks using YAML anchors.
- **Task chain graphs**\
  Per task chain, you can specify a list of other task chains that are required as a prerequisite for this one to run. This can be used to build more complex graphs of tasks. All task chains, are collected in a recursive manner and deduped. They are executed in a leaf-first fashion in which the first stages of execution contain task chains with no preconditions, moving forwards through task chains containing preconditions that already run, finally leading to the entire graph being executed. Use `neomake -e describe -c ...` to view the task chains and the stages (in order) they are executed in.
- **Invocation matrices**\
  Invoke task chains many times by specifying multiple entries in a matrix that's used for parameterizing a seperate chain execution. This feature is heavily inspired by the GitLab pipeline's parallel matrix builds.
- **YAML**\
  No need for any fancy configuration formats or syntax. The entire configuration is done in an easy to understand `yaml` file, including support for handy features such as YAML anchors (and everything in the YAML 1.2 standard).
- **Customizable environment**\
  You can customize which shell or program (such as python) `neomake` uses as interpreter for the command. You can also specify arguments that are provided per invocation via the command line, working directories and environment variables on multiple different levels. Generally, the most inner scope will extend or replace the outer scope.

## Installation

`neomake` is distributed through `cargo`.

1) `cargo install neomake`

## Usage

- `neomake init`
- `neomake run -c test -c othertest -a args.test="some argument"`
- `neomake -e describe -c test -c othertest -o yaml`

## Example configuration

```yaml
version: 0.2

env:
  DEFAULT_ENV_VAR: default var
  OVERRIDE_ENV_VAR_0: old e0
  OVERRIDE_ENV_VAR_1: old e1

.anchor: &anchor |
  printf "test anchor"

chains:
  python:
    description: This is an example of using multiple execution environments (shell and python).
    shell:
      program: bash
      args:
        - -c
    tasks:
      - shell:
          program: python
          args:
            - -c
        script: print('yada')
      - script: "printf test"
      - script: *anchor

  a:
    matrix:
      - env:
          VALUE: a 0
      - env:
          VALUE: a 1
    tasks:
      - script: echo "$VALUE"
  b:
    pre:
      - a
    tasks:
      - script: echo "b"
  c:
    pre:
      - b
    tasks:
      - script: echo "c"

  minimal:
    tasks:
      - script: echo "minimal"

  graph:
    pre:
      - minimal
      - a
      - b
    tasks: []

  test:
    matrix:
      - env:
          OVERRIDE_ENV_VAR_0: new e0
    tasks:
      - env:
          OVERRIDE_ENV_VAR_1: new e1
        script: |
          set -e

          echo "$DEFAULT_ENV_VAR"
          sleep 1
          echo "$OVERRIDE_ENV_VAR_0"
          sleep 1
          echo "$OVERRIDE_ENV_VAR_1"
          sleep 1
          echo "A"
          sleep 1
          echo "B"
          sleep 1
          echo "C"
          sleep 1
          echo "D"
          sleep 1
          echo "{{ args.test }}" # this will require an argument to be passed via '-a args.test="some-argument"'
          sleep 1
          unknown-command
          echo "too far!"

```

## Why

Why would someone build a task runner if there's many alternatives out there? A few of the most well known task running utilities / frameworks are (non exhaustive):

* `make` (`Makefile`) - the original as well as many different implementations
* `Earthly` (`Earthfile`) - executing tasks inside of containers
* `pyinvoke` (`tasks.py`) - executing tasks from within python scripts

I built this utility because all of the alternatives I have tried, including the ones listed above were lacking some features. I was basically looking for a subset of the functionality which the GitLab pipelines provide incl. features such as matrix builds and more. Especially things like invoking commands in many locations, parallelizing tasks, easy parameterization and a few more.
