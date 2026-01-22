import type {} from "@atcute/lexicons";
import * as v from "@atcute/lexicons/validations";
import type {} from "@atcute/lexicons/ambient";

const _mainSchema = /*#__PURE__*/ v.record(
  /*#__PURE__*/ v.literal("self"),
  /*#__PURE__*/ v.object({
    $type: /*#__PURE__*/ v.literal("net.gifdex.actor.profile"),
    /**
     * @accept image/png, image/jpeg
     * @maxSize 300000
     */
    avatar: /*#__PURE__*/ v.optional(/*#__PURE__*/ v.blob()),
    createdAt: /*#__PURE__*/ v.datetimeString(),
    /**
     * @maxGraphemes 64
     */
    displayName: /*#__PURE__*/ v.optional(
      /*#__PURE__*/ v.constrain(/*#__PURE__*/ v.string(), [
        /*#__PURE__*/ v.stringGraphemes(0, 64),
      ]),
    ),
    /**
     * @maxGraphemes 20
     */
    pronouns: /*#__PURE__*/ v.optional(
      /*#__PURE__*/ v.constrain(/*#__PURE__*/ v.string(), [
        /*#__PURE__*/ v.stringGraphemes(0, 20),
      ]),
    ),
  }),
);

type main$schematype = typeof _mainSchema;

export interface mainSchema extends main$schematype {}

export const mainSchema = _mainSchema as mainSchema;

export interface Main extends v.InferInput<typeof mainSchema> {}

declare module "@atcute/lexicons/ambient" {
  interface Records {
    "net.gifdex.actor.profile": mainSchema;
  }
}
