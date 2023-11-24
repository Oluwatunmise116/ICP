import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export type Error = { 'NotFound' : { 'msg' : string } };
export interface Market {
  'id' : bigint,
  'categories' : string,
  'updated_at' : [] | [bigint],
  'product_id' : bigint,
  'created_at' : bigint,
  'seller' : string,
  'product_name' : string,
  'price' : bigint,
}
export type Result = { 'Ok' : Market } |
  { 'Err' : Error };
export interface _SERVICE {
  'delete_market' : ActorMethod<[bigint], Result>,
  'get_market' : ActorMethod<[bigint], Result>,
}
