import { useMutation, useQuery } from "@tanstack/react-query";
import { Plus } from "lucide-react";
import { SearchableAddressSelect } from "@/components/SearchableAddressSelect";
import { Button } from "@/components/ui/Button";
import { coinQueryKey } from "@/lib/coin/profile";
import { rpcGetNewAddress, rpcListAddressGroupings } from "@/lib/rpc/client";

const VERIUM = "verium" as const;

export function PoolPayoutAddressControls({
  address,
  disabled,
  onAddressChange,
}: {
  address: string;
  disabled?: boolean;
  onAddressChange: (address: string) => void;
}) {
  const addresses = useQuery({
    queryKey: coinQueryKey(VERIUM, "listaddressgroupings"),
    queryFn: () => rpcListAddressGroupings(VERIUM),
    staleTime: 30_000,
  });

  const knownAddresses = addresses.data ?? [];

  const newAddress = useMutation({
    mutationFn: () => rpcGetNewAddress(VERIUM),
    onSuccess: (addr) => onAddressChange(addr),
  });

  return (
    <label className="flex flex-col gap-1 text-sm">
      <span className="text-fg-muted">Payout address</span>
      <div className="flex flex-wrap items-start gap-2">
        <SearchableAddressSelect
          value={address}
          addresses={knownAddresses}
          disabled={disabled}
          placeholder="Search addresses…"
          emptyLabel="Select a receive address below"
          onChange={onAddressChange}
          aria-label="Wallet address for pool payouts"
        />
        <Button
          type="button"
          variant="secondary"
          size="sm"
          disabled={disabled || newAddress.isPending}
          onClick={() => newAddress.mutate()}
          className="shrink-0"
        >
          <Plus className="mr-1 h-3.5 w-3.5" />
          New
        </Button>
      </div>
      <span className="text-xs text-fg-subtle">
        Pool rewards are sent to this address. The worker name below is only a
        label for stats.
      </span>
      {!address.trim() ? (
        <p className="text-xs text-warning">
          Choose a payout address before starting pool mining.
        </p>
      ) : null}
    </label>
  );
}
