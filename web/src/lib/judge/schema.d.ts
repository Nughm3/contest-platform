/* eslint-disable */
/**
 * This file was automatically generated by json-schema-to-typescript.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run json-schema-to-typescript to regenerate this file.
 */

export type Message =
  | {
      tests: number;
      type: "Queued";
      [k: string]: unknown;
    }
  | {
      type: "Compiling";
      [k: string]: unknown;
    }
  | {
      exit_code: number;
      stderr: string;
      type: "Compiled";
      [k: string]: unknown;
    }
  | {
      type: "Judging";
      verdict: Verdict;
      [k: string]: unknown;
    }
  | {
      estimated_count: number;
      type: "Skipping";
      [k: string]: unknown;
    }
  | {
      report: Report;
      type: "Done";
      [k: string]: unknown;
    }
  | {
      reason: string;
      type: "Error";
      [k: string]: unknown;
    };
export type Verdict =
  | "CompileError"
  | "RuntimeError"
  | "MemoryLimitExceeded"
  | "TimeLimitExceeded"
  | "WrongAnswer"
  | "Skipped"
  | "Accepted";

export interface Report {
  subtasks: Verdict[];
  task: Verdict;
  tests: TestReport[][];
  [k: string]: unknown;
}
export interface TestReport {
  resource_usage: ResourceUsage;
  verdict: Verdict;
  [k: string]: unknown;
}
export interface ResourceUsage {
  /**
   * Memory usage (bytes)
   */
  memory: number;
  /**
   * System time
   */
  "sys-time": Duration;
  /**
   * User time
   */
  "user-time": Duration;
  [k: string]: unknown;
}
export interface Duration {
  nanos: number;
  secs: number;
  [k: string]: unknown;
}
