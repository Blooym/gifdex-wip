import type {} from "@atcute/lexicons";
import * as v from "@atcute/lexicons/validations";
import type {} from "@atcute/lexicons/ambient";
import * as NetGifdexFeedDefs from "./defs.js";

const _mainSchema = /*#__PURE__*/ v.query("net.gifdex.feed.getPostsByQuery", {
  params: /*#__PURE__*/ v.object({
    actor: /*#__PURE__*/ v.didString(),
    cursor: /*#__PURE__*/ v.optional(/*#__PURE__*/ v.integer()),
    /**
     * @minimum 1
     * @maximum 100
     * @default 50
     */
    limit: /*#__PURE__*/ v.optional(
      /*#__PURE__*/ v.constrain(/*#__PURE__*/ v.integer(), [
        /*#__PURE__*/ v.integerRange(1, 100),
      ]),
      50,
    ),
    /**
     * @maxGraphemes 500
     */
    query: /*#__PURE__*/ v.optional(
      /*#__PURE__*/ v.constrain(/*#__PURE__*/ v.string(), [
        /*#__PURE__*/ v.stringGraphemes(0, 500),
      ]),
    ),
    /**
     * @default "relevance"
     */
    sortBy: /*#__PURE__*/ v.optional(
      /*#__PURE__*/ v.literalEnum(["newest", "oldest", "relevance", "top"]),
      "relevance",
    ),
  }),
  output: {
    type: "lex",
    schema: /*#__PURE__*/ v.object({
      cursor: /*#__PURE__*/ v.optional(/*#__PURE__*/ v.integer()),
      get feed() {
        return /*#__PURE__*/ v.array(NetGifdexFeedDefs.postFeedViewSchema);
      },
    }),
  },
});

type main$schematype = typeof _mainSchema;

export interface mainSchema extends main$schematype {}

export const mainSchema = _mainSchema as mainSchema;

export interface $params extends v.InferInput<mainSchema["params"]> {}
export interface $output extends v.InferXRPCBodyInput<mainSchema["output"]> {}

declare module "@atcute/lexicons/ambient" {
  interface XRPCQueries {
    "net.gifdex.feed.getPostsByQuery": mainSchema;
  }
}
