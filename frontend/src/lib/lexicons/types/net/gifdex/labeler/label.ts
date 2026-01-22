import type {} from "@atcute/lexicons";
import * as v from "@atcute/lexicons/validations";
import type {} from "@atcute/lexicons/ambient";

const _mainSchema = /*#__PURE__*/ v.record(
  /*#__PURE__*/ v.tidString(),
  /*#__PURE__*/ v.object({
    $type: /*#__PURE__*/ v.literal("net.gifdex.labeler.label"),
    createdAt: /*#__PURE__*/ v.datetimeString(),
    /**
     * Optional expiration time for this label
     */
    expiresAt: /*#__PURE__*/ v.optional(/*#__PURE__*/ v.datetimeString()),
    /**
     * Optional context for why this label was applied
     * @maxGraphemes 200
     */
    reason: /*#__PURE__*/ v.optional(
      /*#__PURE__*/ v.constrain(/*#__PURE__*/ v.string(), [
        /*#__PURE__*/ v.stringGraphemes(0, 200),
      ]),
    ),
    /**
     * The rkey of the rule being applied (from net.gifdex.labeler.rule)
     */
    rule: /*#__PURE__*/ v.resourceUriString(),
    /**
     * The content or account being labeled
     */
    subject: /*#__PURE__*/ v.resourceUriString(),
  }),
);

type main$schematype = typeof _mainSchema;

export interface mainSchema extends main$schematype {}

export const mainSchema = _mainSchema as mainSchema;

export interface Main extends v.InferInput<typeof mainSchema> {}

declare module "@atcute/lexicons/ambient" {
  interface Records {
    "net.gifdex.labeler.label": mainSchema;
  }
}
