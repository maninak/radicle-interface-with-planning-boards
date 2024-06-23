import type { ZodSchema, z } from "zod";

import { array, literal, number, object, string, union } from "zod";

export interface SuccessResponse {
  success: true;
}

export const successResponseSchema = object({
  success: literal(true),
}) satisfies ZodSchema<SuccessResponse>;

const policySchema = union([literal("allow"), literal("block")]);
const scopeSchema = union([literal("followed"), literal("all")]);

export const nodeConfigSchema = object({
  alias: string(),
  peers: union([
    object({ type: literal("static") }),
    object({ type: literal("dynamic") }),
  ]),
  listen: array(string()),
  connect: array(string()),
  externalAddresses: array(string()),
  proxy: string().optional(),
  onion: union([
    object({
      mode: literal("proxy"),
      address: string(),
    }),
    object({ mode: literal("forward") }),
  ]).optional(),
  log: union([
    literal("ERROR"),
    literal("WARN"),
    literal("INFO"),
    literal("DEBUG"),
    literal("TRACE"),
  ]),
  network: union([literal("main"), literal("test")]),
  relay: union([literal("always"), literal("never"), literal("auto")]),
  limits: object({
    routingMaxSize: number(),
    routingMaxAge: number(),
    fetchConcurrency: number(),
    gossipMaxAge: number(),
    maxOpenFiles: number(),
    rate: object({
      inbound: object({
        fillRate: number(),
        capacity: number(),
      }),
      outbound: object({
        fillRate: number(),
        capacity: number(),
      }),
    }),
    connection: object({
      inbound: number(),
      outbound: number(),
    }),
  }),
  workers: number(),
  seedingPolicy: object({
    default: policySchema,
    scope: scopeSchema.optional(),
  }),
});

export type Policy = z.infer<typeof policySchema>;
export type Scope = z.infer<typeof scopeSchema>;

export const rangeSchema = union([
  object({
    type: literal("lines"),
    range: object({ start: number(), end: number() }),
  }),
  object({
    type: literal("chars"),
    line: number(),
    range: object({ start: number(), end: number() }),
  }),
]);

export type Range = z.infer<typeof rangeSchema>;

export const codeLocationSchema = object({
  commit: string(),
  path: string(),
  old: rangeSchema.nullable(),
  new: rangeSchema.nullable(),
});

export type CodeLocation = z.infer<typeof codeLocationSchema>;

export const authorSchema = object({
  id: string(),
  alias: string().optional(),
});
