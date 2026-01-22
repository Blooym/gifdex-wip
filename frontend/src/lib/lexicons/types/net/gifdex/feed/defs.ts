import type {} from "@atcute/lexicons";
import * as v from "@atcute/lexicons/validations";
import * as NetGifdexActorDefs from "../actor/defs.js";

const _postFeedViewSchema = /*#__PURE__*/ v.object({
  $type: /*#__PURE__*/ v.optional(
    /*#__PURE__*/ v.literal("net.gifdex.feed.defs#postFeedView"),
  ),
  get author() {
    return NetGifdexActorDefs.profileViewBasicSchema;
  },
  createdAt: /*#__PURE__*/ v.datetimeString(),
  editedAt: /*#__PURE__*/ v.optional(/*#__PURE__*/ v.datetimeString()),
  favouriteCount: /*#__PURE__*/ v.integer(),
  indexedAt: /*#__PURE__*/ v.datetimeString(),
  languages: /*#__PURE__*/ v.optional(
    /*#__PURE__*/ v.array(/*#__PURE__*/ v.string()),
  ),
  get media() {
    return postViewMediaSchema;
  },
  tags: /*#__PURE__*/ v.optional(
    /*#__PURE__*/ v.array(/*#__PURE__*/ v.string()),
  ),
  title: /*#__PURE__*/ v.string(),
  uri: /*#__PURE__*/ v.resourceUriString(),
  get viewer() {
    return viewerStateSchema;
  },
});
const _postViewSchema = /*#__PURE__*/ v.object({
  $type: /*#__PURE__*/ v.optional(
    /*#__PURE__*/ v.literal("net.gifdex.feed.defs#postView"),
  ),
  get author() {
    return NetGifdexActorDefs.profileViewBasicSchema;
  },
  createdAt: /*#__PURE__*/ v.datetimeString(),
  editedAt: /*#__PURE__*/ v.optional(/*#__PURE__*/ v.datetimeString()),
  favouriteCount: /*#__PURE__*/ v.integer(),
  indexedAt: /*#__PURE__*/ v.datetimeString(),
  languages: /*#__PURE__*/ v.optional(
    /*#__PURE__*/ v.array(/*#__PURE__*/ v.string()),
  ),
  get media() {
    return postViewMediaSchema;
  },
  tags: /*#__PURE__*/ v.optional(
    /*#__PURE__*/ v.array(/*#__PURE__*/ v.string()),
  ),
  title: /*#__PURE__*/ v.string(),
  uri: /*#__PURE__*/ v.resourceUriString(),
  get viewer() {
    return viewerStateSchema;
  },
});
const _postViewMediaSchema = /*#__PURE__*/ v.object({
  $type: /*#__PURE__*/ v.optional(
    /*#__PURE__*/ v.literal("net.gifdex.feed.defs#postViewMedia"),
  ),
  alt: /*#__PURE__*/ v.optional(/*#__PURE__*/ v.string()),
  get dimensions() {
    return postViewMediaDimensionsSchema;
  },
  fullsizeUrl: /*#__PURE__*/ v.genericUriString(),
  mimeType: /*#__PURE__*/ v.string(),
  thumbnailUrl: /*#__PURE__*/ v.genericUriString(),
});
const _postViewMediaDimensionsSchema = /*#__PURE__*/ v.object({
  $type: /*#__PURE__*/ v.optional(
    /*#__PURE__*/ v.literal("net.gifdex.feed.defs#postViewMediaDimensions"),
  ),
  height: /*#__PURE__*/ v.integer(),
  width: /*#__PURE__*/ v.integer(),
});
const _viewerStateSchema = /*#__PURE__*/ v.object({
  $type: /*#__PURE__*/ v.optional(
    /*#__PURE__*/ v.literal("net.gifdex.feed.defs#viewerState"),
  ),
  favourite: /*#__PURE__*/ v.optional(/*#__PURE__*/ v.tidString()),
});

type postFeedView$schematype = typeof _postFeedViewSchema;
type postView$schematype = typeof _postViewSchema;
type postViewMedia$schematype = typeof _postViewMediaSchema;
type postViewMediaDimensions$schematype = typeof _postViewMediaDimensionsSchema;
type viewerState$schematype = typeof _viewerStateSchema;

export interface postFeedViewSchema extends postFeedView$schematype {}
export interface postViewSchema extends postView$schematype {}
export interface postViewMediaSchema extends postViewMedia$schematype {}
export interface postViewMediaDimensionsSchema extends postViewMediaDimensions$schematype {}
export interface viewerStateSchema extends viewerState$schematype {}

export const postFeedViewSchema = _postFeedViewSchema as postFeedViewSchema;
export const postViewSchema = _postViewSchema as postViewSchema;
export const postViewMediaSchema = _postViewMediaSchema as postViewMediaSchema;
export const postViewMediaDimensionsSchema =
  _postViewMediaDimensionsSchema as postViewMediaDimensionsSchema;
export const viewerStateSchema = _viewerStateSchema as viewerStateSchema;

export interface PostFeedView extends v.InferInput<typeof postFeedViewSchema> {}
export interface PostView extends v.InferInput<typeof postViewSchema> {}
export interface PostViewMedia extends v.InferInput<
  typeof postViewMediaSchema
> {}
export interface PostViewMediaDimensions extends v.InferInput<
  typeof postViewMediaDimensionsSchema
> {}
export interface ViewerState extends v.InferInput<typeof viewerStateSchema> {}
