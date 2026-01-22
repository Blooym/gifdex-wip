import type {} from "@atcute/lexicons";
import * as v from "@atcute/lexicons/validations";
import type {} from "@atcute/lexicons/ambient";

const _mainSchema = /*#__PURE__*/ v.record(
  /*#__PURE__*/ v.string(),
  /*#__PURE__*/ v.object({
    $type: /*#__PURE__*/ v.literal("net.gifdex.feed.post"),
    /**
     * Client-declared timestamp when this post was originally created.
     */
    createdAt: /*#__PURE__*/ v.datetimeString(),
    /**
     * Indicates human language of post content, including title and the media itself.
     * @maxLength 3
     */
    languages: /*#__PURE__*/ v.optional(
      /*#__PURE__*/ v.constrain(
        /*#__PURE__*/ v.array(/*#__PURE__*/ v.languageCodeString()),
        [/*#__PURE__*/ v.arrayLength(0, 3)],
      ),
    ),
    get media() {
      return postMediaSchema;
    },
    /**
     * Tags that apply to the content of the post, used for discoverability.
     * @maxLength 5
     */
    tags: /*#__PURE__*/ v.optional(
      /*#__PURE__*/ v.constrain(
        /*#__PURE__*/ v.array(
          /*#__PURE__*/ v.constrain(/*#__PURE__*/ v.string(), [
            /*#__PURE__*/ v.stringGraphemes(0, 40),
          ]),
        ),
        [/*#__PURE__*/ v.arrayLength(0, 5)],
      ),
    ),
    /**
     * The title of the post.
     * @maxGraphemes 80
     */
    title: /*#__PURE__*/ v.constrain(/*#__PURE__*/ v.string(), [
      /*#__PURE__*/ v.stringGraphemes(0, 80),
    ]),
  }),
);
const _postMediaSchema = /*#__PURE__*/ v.object({
  $type: /*#__PURE__*/ v.optional(
    /*#__PURE__*/ v.literal("net.gifdex.feed.post#postMedia"),
  ),
  /**
   * @maxGraphemes 5000
   */
  alt: /*#__PURE__*/ v.optional(
    /*#__PURE__*/ v.constrain(/*#__PURE__*/ v.string(), [
      /*#__PURE__*/ v.stringGraphemes(0, 5000),
    ]),
  ),
  /**
   * @accept image/webp
   * @maxSize 5000000
   */
  blob: /*#__PURE__*/ v.blob(),
});

type main$schematype = typeof _mainSchema;
type postMedia$schematype = typeof _postMediaSchema;

export interface mainSchema extends main$schematype {}
export interface postMediaSchema extends postMedia$schematype {}

export const mainSchema = _mainSchema as mainSchema;
export const postMediaSchema = _postMediaSchema as postMediaSchema;

export interface Main extends v.InferInput<typeof mainSchema> {}
export interface PostMedia extends v.InferInput<typeof postMediaSchema> {}

declare module "@atcute/lexicons/ambient" {
  interface Records {
    "net.gifdex.feed.post": mainSchema;
  }
}
