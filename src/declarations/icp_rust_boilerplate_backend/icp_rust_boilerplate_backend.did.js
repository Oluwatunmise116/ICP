export const idlFactory = ({ IDL }) => {
  const Market = IDL.Record({
    'id' : IDL.Nat64,
    'categories' : IDL.Text,
    'updated_at' : IDL.Opt(IDL.Nat64),
    'product_id' : IDL.Nat64,
    'created_at' : IDL.Nat64,
    'seller' : IDL.Text,
    'product_name' : IDL.Text,
    'price' : IDL.Nat64,
  });
  const Error = IDL.Variant({ 'NotFound' : IDL.Record({ 'msg' : IDL.Text }) });
  const Result = IDL.Variant({ 'Ok' : Market, 'Err' : Error });
  return IDL.Service({
    'delete_market' : IDL.Func([IDL.Nat64], [Result], []),
    'get_market' : IDL.Func([IDL.Nat64], [Result], ['query']),
  });
};
export const init = ({ IDL }) => { return []; };
