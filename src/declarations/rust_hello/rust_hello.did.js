export const idlFactory = ({ IDL }) => {
  return IDL.Service({
    'verify' : IDL.Func([IDL.Text, IDL.Text, IDL.Text], [IDL.Bool], ['query']),
  });
};
export const init = ({ IDL }) => { return []; };
