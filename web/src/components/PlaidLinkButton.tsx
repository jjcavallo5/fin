import { usePlaidLink } from "react-plaid-link";

const PlaidLinkButton = ({ token }: { token: string }) => {
  const onSuccess = () => {
    // Do Stuff
  };
  const { open, ready } = usePlaidLink({ token, onSuccess });

  return (
    <button
      onClick={() => open()}
      className="w-[772px] rounded-sm border-[1px] border-bg-muted hover:bg-accent hover:text-bg hover:border-white transition-all cursor-pointer py-4  bg-accent/75 border-accent/50 font-mono text-white"
    >
      LINK_ACCOUNT &rarr;
    </button>
  );
};

export default PlaidLinkButton;
