# Turns Scheduler

A flexible, configuration-driven, command-line tool for generating on-call schedules.

## Overview

This tool reads a YAML configuration file to understand the people involved, their availability (including Out-of-Office periods), and their preferences. It then uses a chosen scheduling algorithm to generate a fair and balanced on-call schedule, printing the result to the console.

## Features

- **Declarative Configuration**: Define your entire schedule in a simple YAML file.
- **Multiple Scheduling Algorithms**: Choose from different scheduling strategies to best suit your team's needs.
- **Out-of-Office (OOO) Handling**: Ensures people are not scheduled for on-call duties when they are unavailable.
- **Preference-Based Scheduling**: Allows individuals to specify dates they `Want` or `NotWant` to be on-call.
- **Load Balancing**: The more advanced algorithms work to distribute the on-call load as evenly as possible among team members.

## Configuration

The schedule is defined in a YAML file (e.g., `turns.yaml`).

### Example: `turns.yaml`

```yaml
people:
  alice:
    name: Alice
    ooo:
      - !Day 2025-09-15
      - !Period
        from: 2025-08-23
        to: 2025-09-07
  bob:
    name: Bob
    preferences:
      - !Want 2025-09-06
      - !NotWant 2025-09-01
  charlie:
    name: Charlie

schedule:
  from: 2025-08-01
  to: 2025-09-30
  algo: !Balanced
    min_turn_days: 3
    max_turn_days: 10
```

### Structure

- **`people`**: A map of individuals. Each person has:
    - `name`: The person's display name.
    - `ooo` (optional): A list of dates or periods they are unavailable.
        - `!Day YYYY-MM-DD`: A single day.
        - `!Period { from: YYYY-MM-DD, to: YYYY-MM-DD }`: A date range.
    - `preferences` (optional): A list of scheduling preferences.
        - `!Want YYYY-MM-DD`: A preferred on-call date.
        - `!NotWant YYYY-MM-DD`: A date the person wishes to avoid.
- **`schedule`**: Defines the scheduling parameters.
    - `from`: The start date of the schedule.
    - `to`: The end date of the schedule.
    - `algo`: The scheduling algorithm to use.

## Scheduling Algorithms

You can choose one of the following algorithms in your configuration file:

### 1. Round Robin

`!RoundRobin { turn_length_days: 7 }`

The simplest algorithm. It assigns turns to people in a sequential, rotating order.

- **Pros**: Predictable and easy to understand.
- **Cons**: Does not account for load balancing or preferences.

### 2. Greedy

`!Greedy { turn_length_days: 7, preference_weight: 7 }`

A more advanced algorithm that prioritizes preferences and load balancing. At each step, it chooses the best person for the next turn based on their availability, preferences, and current on-call load.

- **Pros**: Respects preferences and tries to keep the load balanced.
- **Cons**: Can sometimes make locally optimal choices that lead to less balanced schedules over the long term.

### 3. Balanced

`!Balanced { min_turn_days: 3, max_turn_days: 10 }`

The most sophisticated algorithm. It uses variable turn lengths and a lookahead mechanism to find the assignment that results in the most balanced load distribution for the team.

- **Pros**: Produces the most balanced and fair schedules.
- **Cons**: The schedule can be less predictable than `RoundRobin`.

## Usage

### Prerequisites

- [Rust and Cargo](https://www.rust-lang.org/tools/install)

### Running the Scheduler

1.  Clone the repository.
2.  Create or modify the `turns.yaml` configuration file.
3.  Run the application from your terminal:

```bash
cargo run
```

By default, the tool looks for a `turns.yaml` file in the current directory. You can specify a different configuration file using the `--config` flag:

```bash
cargo run -- --config /path/to/your/config.yaml
```
