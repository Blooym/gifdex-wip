import type {} from "@atcute/lexicons";
import * as v from "@atcute/lexicons/validations";
import type {} from "@atcute/lexicons/ambient";

const _annotateSchema = /*#__PURE__*/ v.object({
  $type: /*#__PURE__*/ v.optional(
    /*#__PURE__*/ v.literal("net.gifdex.labeler.rule#annotate"),
  ),
  adultContent: /*#__PURE__*/ v.boolean(),
  defaultSetting: /*#__PURE__*/ v.literalEnum([
    "hide",
    "ignore",
    "inform",
    "warn",
  ]),
});
const _mainSchema = /*#__PURE__*/ v.record(
  /*#__PURE__*/ v.string(),
  /*#__PURE__*/ v.object({
    $type: /*#__PURE__*/ v.literal("net.gifdex.labeler.rule"),
    get behaviour() {
      return /*#__PURE__*/ v.variant([annotateSchema, moderateSchema]);
    },
    createdAt: /*#__PURE__*/ v.datetimeString(),
    /**
     * @maxGraphemes 200
     */
    description: /*#__PURE__*/ v.constrain(/*#__PURE__*/ v.string(), [
      /*#__PURE__*/ v.stringGraphemes(0, 200),
    ]),
    /**
     * @maxGraphemes 20
     */
    name: /*#__PURE__*/ v.constrain(/*#__PURE__*/ v.string(), [
      /*#__PURE__*/ v.stringGraphemes(0, 20),
    ]),
    selfLabel: /*#__PURE__*/ v.optional(/*#__PURE__*/ v.boolean()),
  }),
);
const _moderateSchema = /*#__PURE__*/ v.object({
  $type: /*#__PURE__*/ v.optional(
    /*#__PURE__*/ v.literal("net.gifdex.labeler.rule#moderate"),
  ),
  takedown: /*#__PURE__*/ v.boolean(),
});

type annotate$schematype = typeof _annotateSchema;
type main$schematype = typeof _mainSchema;
type moderate$schematype = typeof _moderateSchema;

export interface annotateSchema extends annotate$schematype {}
export interface mainSchema extends main$schematype {}
export interface moderateSchema extends moderate$schematype {}

export const annotateSchema = _annotateSchema as annotateSchema;
export const mainSchema = _mainSchema as mainSchema;
export const moderateSchema = _moderateSchema as moderateSchema;

export interface Annotate extends v.InferInput<typeof annotateSchema> {}
export interface Main extends v.InferInput<typeof mainSchema> {}
export interface Moderate extends v.InferInput<typeof moderateSchema> {}

declare module "@atcute/lexicons/ambient" {
  interface Records {
    "net.gifdex.labeler.rule": mainSchema;
  }
}
