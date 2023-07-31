# neomake

`neomake` is a task runner CLI utility that acts as a modern alternative to known utilities like `Makefile`s.

## Project state

`neomake` is released and  **stable**. It is actively maintained and used in production.

## Features

- **DAG execution**\
  Tasks are run in nodes. Nodes can be chained together to create a DAG. Simply specify all the nodes you want executed and it will automagically create the DAG based on the defined dependencies.
- **Parallel task execution**\
  The DAG generations are called stages. Stages are executed in sequence while all tasks inside of the stages are executed in parallel. Workloads are executed in OS threads. The default size of the threadpool is 1 but can be configured.
- **Matrix invocations**\
  Specify n-dimensional matrices that are used to invoke the node many times. You can define dense and sparse matrices. The node will be executed for every element in the cartesion product of the matrix.
- **YAML**\
  No need for any fancy configuration formats or syntax. The entire configuration is done in an easy to understand `yaml` file, including support for handy features such as YAML anchors (and everything in the `YAML 1.2` standard).
- **Customizable environment**\
  You can customize which shell or program (such as bash or python) `neomake` uses as interpreter for the command. You can also specify arguments that are provided per invocation via the command line, working directories and environment variables on multiple different levels. Generally, values defined in the inner scope will extend and replace the outer scope.
- **Plan & execute**\
  Supporting execution of commands in two stages. First plan and render the entire execution. Then invoke the execution engine with the plan. This way, plans can be stored and reviewed before execution.

## Installation

`neomake` is distributed through `cargo`.

1) For the latest stable version:\
  `cargo install neomake`
2) For the bleeding edge master branch:\
  `cargo install --git https://github.com/replicadse/neomake.git`

## Example

First, initialize an example workflow file with the following command.

```bash
neomake workflow init -tpython
```

Now, execute the `count` node. Per default, `neomake` will only use exactly one worker thread and execute the endless embedded python program.

```bash
neomake plan -ccount | neomake x
```

In order to work on all 4 desired executions (defined as 2x2 matrix), call neomake with the number of worker threads desired. Now you will see that the 4 programs will be executed in parallel.

```bash
neomake plan -ccount | neomake x -w4
```

## Graph execution

Execute nodes as follows.

```bash
neomake plan -f ./test/.neomake.yaml -c bravo -c charlie -oron | neomake execute -fron
```

Nodes can define an array of dependenies (other nodes) that need to be executed beforehand. All node executions are deduplicated so that every node is only executed exactly once if requested for invocation or as a prerequisite on any level to any node that is to be executed. Alongside the ability to specify multiple node to be executed per command line call, this feature allows for complex workflows to be executed.\
Let's assume the following graph of nodes and their dependencies:

```bash
neomake ls
```

```yaml
---
nodes:
  - name: A
  - name: B
  - name: C
    pre:
      - A
  - name: D
    pre:
      - B
  - name: E
    pre:
      - A
      - D
```

In words, `A` and `B` are nodes without any prerequisites whereas `C` depends on `A` and `D` depends on `B`. Notably, `E` depends on both `A` and `D`. This means that `E` also transiently depends on any dependencies of `A` (`{}`) and `D` (`{B}`).

It is also possible to get a simple description of the workflow to be executed.
```bash
neomake describe -cC -cE
```

```yaml
---
stages:
  - - A
    - B
  - - D
  - - E
```

Stages need to run sequentially due to their nodes dependency on nodes executed in a previous stage. Tasks inside a stage are run in parallel (in an OS thread pool of the size given to the `worker` argument). `neomake` is also able to identify and prevent recursions in the execution graph and will fail if the execution of such a sub graph is attempted.

## Why

Why would someone build a task runner if there's many alternatives out there? A few of the most well known task running utilities / frameworks are (non exhaustive):

* `make` (`Makefile`) - the original as well as many different implementations
* `Earthly` (`Earthfile`) - executing tasks inside of containers
* `pyinvoke` (`tasks.py`) - executing tasks from within python scripts

I built this utility because all of the alternatives I have tried, including the ones listed above were lacking some features. I was basically looking for a subset of the functionality which the GitLab pipelines provide incl. features such as matrix builds and more. Especially things like invoking commands in many locations, parallelizing tasks, easy parameterization and a few more.

## Example configuration

```yaml
<-- ../res/templates/max.neomake.yaml -->
```

For more examples, call `neomake workflow init --help` or look at the schema with `neomake workflow schema`.

## Schema

```json
<-- ./schema.json -->
```
