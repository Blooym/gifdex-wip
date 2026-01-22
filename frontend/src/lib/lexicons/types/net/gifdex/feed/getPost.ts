import type {} from "@atcute/lexicons";
import * as v from "@atcute/lexicons/validations";
import type {} from "@atcute/lexicons/ambient";
import * as NetGifdexFeedDefs from "./defs.js";

const _mainSchema = /*#__PURE__*/ v.query("net.gifdex.feed.getPost", {
  params: /*#__PURE__*/ v.object({
    actor: /*#__PURE__*/ v.didString(),
    rkey: /*#__PURE__*/ v.string(),
  }),
  output: {
    type: "lex",
    schema: /*#__PURE__*/ v.object({
      get post() {
        return NetGifdexFeedDefs.postViewSchema;
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
    "net.gifdex.feed.getPost": mainSchema;
  }
}
