import type {} from "@atcute/lexicons";
import * as v from "@atcute/lexicons/validations";
import type {} from "@atcute/lexicons/ambient";
import * as NetGifdexActorDefs from "./defs.js";

const _mainSchema = /*#__PURE__*/ v.query("net.gifdex.actor.getProfile", {
  params: /*#__PURE__*/ v.object({
    actor: /*#__PURE__*/ v.didString(),
  }),
  output: {
    type: "lex",
    get schema() {
      return NetGifdexActorDefs.profileViewSchema;
    },
  },
});

type main$schematype = typeof _mainSchema;

export interface mainSchema extends main$schematype {}

export const mainSchema = _mainSchema as mainSchema;

export interface $params extends v.InferInput<mainSchema["params"]> {}
export type $output = v.InferXRPCBodyInput<mainSchema["output"]>;

declare module "@atcute/lexicons/ambient" {
  interface XRPCQueries {
    "net.gifdex.actor.getProfile": mainSchema;
  }
}
