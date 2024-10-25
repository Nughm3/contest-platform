/* eslint-disable */
/**
 * This file was automatically generated by json-schema-to-typescript.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run json-schema-to-typescript to regenerate this file.
 */

export type Difficulty = "Easy" | "Medium" | "Hard";

export interface Contest {
  duration: number;
  judge: Config;
  name: string;
  page: string;
  "submission-cooldown": number;
  tasks: Task[];
  [k: string]: unknown;
}
export interface Config {
  languages: Language[];
  "resource-limits": ResourceLimits;
  "skip-count": number;
  [k: string]: unknown;
}
export interface Language {
  compile?: string[] | null;
  filename: string;
  name: string;
  run: string[];
  [k: string]: unknown;
}
export interface ResourceLimits {
  /**
   * CPU time (seconds)
   */
  cpu: number;
  /**
   * CPU time tolerance (seconds)
   */
  "cpu-tolerance": number;
  /**
   * Memory usage (bytes)
   */
  memory: number;
  /**
   * Memory usage tolerance (bytes)
   */
  "memory-tolerance": number;
  [k: string]: unknown;
}
export interface Task {
  difficulty: Difficulty;
  name: string;
  page: string;
  subtasks: Subtask[];
  [k: string]: unknown;
}
export interface Subtask {
  tests: Test[];
  [k: string]: unknown;
}
export interface Test {
  input: string;
  output: string;
  [k: string]: unknown;
}