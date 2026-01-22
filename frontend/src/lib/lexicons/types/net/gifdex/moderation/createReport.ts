import type {} from "@atcute/lexicons";
import * as v from "@atcute/lexicons/validations";
import type {} from "@atcute/lexicons/ambient";

const _mainSchema = /*#__PURE__*/ v.procedure(
  "net.gifdex.moderation.createReport",
  {
    params: null,
    input: {
      type: "lex",
      schema: /*#__PURE__*/ v.object({
        /**
         * @maxGraphemes 2000
         */
        reason: /*#__PURE__*/ v.optional(
          /*#__PURE__*/ v.constrain(/*#__PURE__*/ v.string(), [
            /*#__PURE__*/ v.stringGraphemes(0, 2000),
          ]),
        ),
        reasonCategory: /*#__PURE__*/ v.optional(
          /*#__PURE__*/ v.literalEnum(["hate", "other", "sexual", "spam"]),
        ),
        subject: /*#__PURE__*/ v.optional(/*#__PURE__*/ v.resourceUriString()),
      }),
    },
    output: {
      type: "lex",
      schema: /*#__PURE__*/ v.object({}),
    },
  },
);

type main$schematype = typeof _mainSchema;

export interface mainSchema extends main$schematype {}

export const mainSchema = _mainSchema as mainSchema;

export interface $params {}
export interface $input extends v.InferXRPCBodyInput<mainSchema["input"]> {}
export interface $output extends v.InferXRPCBodyInput<mainSchema["output"]> {}

declare module "@atcute/lexicons/ambient" {
  interface XRPCProcedures {
    "net.gifdex.moderation.createReport": mainSchema;
  }
}
