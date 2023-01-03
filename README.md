# neomake

`neomake` is a fully open source task runner CLI utility that acts as a modern alternative to known utilities.

## State

`neomake` is currently a pre-release and to be considered as **unstable**.

## Features

- **Task chains**\
  Execute many tasks in sequence, easily re-use single tasks using YAML anchors.
- **Invocation matrix**\
  Invoke task chains many times by specifying multiple entries in a matrix that's used for parameterizing a seperate chain execution. This feature is heavily inspired by the GitLab pipeline's matrix builds.
- **YAML**\
  No need for any fancy configuration formats or syntax. The entire configuration is done in an easy to understand `yaml` file, including support for features such as YAML anchors (and everything in the YAML 1.2 standard).
  
## How to use

1. `cargo install neomake`
2. `neomake init`
3. `neomake run -c test -a args.test="some argument"`

## Why

Why would someone build a task runner if there's many alternatives out there? A few of the most well known task running utilities / frameworks are (non exhaustive):

* `make` (`Makefile`) - the original as well as many different implementations
* `Earthly` (`Earthfile`) - executing tasks inside of containers
* `pyinvoke` (`tasks.py`) - executing tasks from within python scripts

I built this utility because all of the alternatives I have tried, including the ones listed above were lacking some features. I was basically looking for a subset of the functionality which the GitLab pipelines provide incl. features such as matrix builds and more. Especially things like invoking commands in many locations, parallelizing tasks, easy parameterization and a few more.
