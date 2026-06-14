import { useActiveCoin } from '@/lib/coin/context';
import { coinQueryKey } from '@/lib/coin/profile';
import { useMemo, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { BookUser, Pencil, Plus, Save, Trash2, X } from 'lucide-react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Badge } from '@/components/ui/Badge';
import { MobileSegmented } from '@/components/mobile/MobileSegmented';
import { useResponsiveLayout } from '@/hooks/useResponsiveLayout';
import {
  deleteAddressBookEntry,
  listAddressBookEntries,
  upsertAddressBookEntry,
  type AddressBookCategory,
  type AddressBookEntry,
} from '@/lib/address-book';
import { cn } from '@/lib/utils';

interface DraftEntry {
  id?: string;
  address: string;
  label: string;
  notes: string;
  category: AddressBookCategory;
}

function emptyDraft(category: AddressBookCategory = 'send'): DraftEntry {
  return { address: '', label: '', notes: '', category };
}

export function AddressBook() {
  const coin = useActiveCoin();
  const { isPhoneLayout } = useResponsiveLayout();
  const queryClient = useQueryClient();
  const [filter, setFilter] = useState<AddressBookCategory>('send');
  const [draft, setDraft] = useState<DraftEntry | null>(null);

  const entries = useQuery({
    queryKey: coinQueryKey(coin, 'address-book'),
    queryFn: () => listAddressBookEntries(coin),
  });

  const upsert = useMutation({
    mutationFn: (entry: DraftEntry) =>
      upsertAddressBookEntry(coin, {
        id: entry.id ?? '',
        address: entry.address.trim(),
        label: entry.label.trim(),
        notes: entry.notes.trim(),
        category: entry.category,
      }),
    onSuccess: (saved) => {
      const category: AddressBookCategory = saved.category === 'receive' ? 'receive' : 'send';
      queryClient.setQueryData<AddressBookEntry[]>(coinQueryKey(coin, 'address-book'), (prev) => {
        const list = prev ?? [];
        const normalized = { ...saved, category };
        const index = list.findIndex((e) => e.id === normalized.id);
        if (index >= 0) {
          const next = [...list];
          next[index] = normalized;
          return next;
        }
        return [...list, normalized];
      });
      setFilter(category);
      setDraft(null);
    },
  });

  const remove = useMutation({
    mutationFn: (id: string) => deleteAddressBookEntry(coin, id),
    onSuccess: (_result, id) => {
      queryClient.setQueryData<AddressBookEntry[]>(coinQueryKey(coin, 'address-book'), (prev) =>
        (prev ?? []).filter((e) => e.id !== id)
      );
    },
  });

  const filtered = useMemo(() => {
    const rows = (entries.data ?? []).filter((e) => e.category === filter);
    return rows.sort((a, b) => a.label.localeCompare(b.label));
  }, [entries.data, filter]);

  if (isPhoneLayout) {
    return (
      <div className="mobile-page mobile-page--address-book">
        <div className="mobile-address-book-controls">
          <section className="mobile-panel rounded-2xl border border-border bg-bg-panel px-4 py-4 shadow-sm">
            <div className="flex items-start justify-between gap-3">
              <div className="min-w-0">
                <h1 className="flex items-center gap-2 text-base font-semibold text-fg">
                  <BookUser className="h-4 w-4 shrink-0 text-accent" />
                  Saved addresses
                </h1>
                <p className="mt-1 text-xs leading-relaxed text-fg-subtle">
                  Contacts for sending and receiving. Stored on this device only.
                </p>
              </div>
            </div>
            <Button
              className="mt-4 h-11 w-full rounded-xl"
              onClick={() => setDraft(emptyDraft(filter))}
            >
              <Plus className="h-4 w-4" /> Add address
            </Button>
          </section>

          <MobileSegmented
            value={filter}
            ariaLabel="Address type"
            onChange={setFilter}
            options={[
              { value: 'send', label: 'Send to' },
              { value: 'receive', label: 'Receive at' },
            ]}
          />

          {draft && (
            <section className="mobile-panel rounded-2xl border border-accent/40 bg-accent/5 p-4">
              <DraftRow
                draft={draft}
                onChange={setDraft}
                onCancel={() => setDraft(null)}
                onSave={() => upsert.mutate(draft)}
                saving={upsert.isPending}
                saveError={upsert.error ? String(upsert.error) : null}
                mobile
              />
            </section>
          )}

          {entries.isError && (
            <div className="mobile-banner border border-danger/30 bg-danger/10 text-danger">
              Could not load address book: {String(entries.error)}
            </div>
          )}
          {upsert.isError && (
            <div className="mobile-banner border border-danger/30 bg-danger/10 text-danger">
              Save failed: {String(upsert.error)}
            </div>
          )}
        </div>

        <div className="mobile-address-book-list">
          {entries.isLoading ? (
            <div className="py-12 text-center text-sm text-fg-muted">Loading…</div>
          ) : filtered.length === 0 ? (
            <section className="mobile-panel rounded-2xl border border-dashed border-border px-4 py-12 text-center">
              <p className="text-sm font-medium text-fg-muted">
                No {filter === 'send' ? 'send' : 'receive'} addresses yet
              </p>
              <p className="mt-1 text-xs text-fg-subtle">
                Tap Add address to save a label and address for quick reuse.
              </p>
            </section>
          ) : (
            <ul className="flex flex-col gap-2.5">
              {filtered.map((entry) => (
                <EntryRow
                  key={entry.id}
                  entry={entry}
                  onEdit={() => setDraft({ ...entry })}
                  onDelete={() => remove.mutate(entry.id)}
                  mobile
                />
              ))}
            </ul>
          )}
        </div>
      </div>
    );
  }

  return (
    <div className="flex min-w-0 max-w-full flex-col gap-4">
      <Card>
        <CardHeader className="flex-row items-center justify-between gap-3">
          <div>
            <CardTitle className="flex items-center gap-2">
              <BookUser className="h-4 w-4 text-accent" /> Address book
            </CardTitle>
            <CardDescription>
              Saved sending and receiving addresses. Stored locally only.
            </CardDescription>
          </div>
          <Button size="sm" onClick={() => setDraft(emptyDraft(filter))}>
            <Plus className="h-3.5 w-3.5" /> New entry
          </Button>
        </CardHeader>
        <CardContent className="flex flex-col gap-3">
          <div className="inline-flex w-fit rounded-md border border-border bg-bg-subtle p-1">
            {(['send', 'receive'] as AddressBookCategory[]).map((cat) => (
              <button
                key={cat}
                type="button"
                onClick={() => setFilter(cat)}
                className={cn(
                  'h-8 rounded px-3 text-xs font-medium capitalize',
                  filter === cat
                    ? 'bg-accent text-accent-fg'
                    : 'text-fg-muted hover:bg-bg-panel hover:text-fg'
                )}
              >
                {cat}
              </button>
            ))}
          </div>

          {draft && (
            <div className="rounded-md border border-accent/40 bg-accent/5 p-3">
              <DraftRow
                draft={draft}
                onChange={setDraft}
                onCancel={() => setDraft(null)}
                onSave={() => upsert.mutate(draft)}
                saving={upsert.isPending}
                saveError={upsert.error ? String(upsert.error) : null}
              />
            </div>
          )}

          {entries.isError && (
            <div className="rounded-md border border-danger/30 bg-danger/10 px-3 py-2 text-xs text-danger">
              Could not load address book: {String(entries.error)}
            </div>
          )}
          {upsert.isError && (
            <div className="rounded-md border border-danger/30 bg-danger/10 px-3 py-2 text-xs text-danger">
              Save failed: {String(upsert.error)}
            </div>
          )}
          {entries.isLoading ? (
            <div className="py-10 text-center text-sm text-fg-muted">Loading…</div>
          ) : filtered.length === 0 ? (
            <div className="py-10 text-center text-sm text-fg-subtle">
              No {filter} addresses yet.
            </div>
          ) : (
            <ul className="flex flex-col gap-2">
              {filtered.map((entry) => (
                <EntryRow
                  key={entry.id}
                  entry={entry}
                  onEdit={() => setDraft({ ...entry })}
                  onDelete={() => remove.mutate(entry.id)}
                />
              ))}
            </ul>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

function DraftRow({
  draft,
  onChange,
  onSave,
  onCancel,
  saving,
  saveError,
  mobile = false,
}: {
  draft: DraftEntry;
  onChange: (next: DraftEntry) => void;
  onSave: () => void;
  onCancel: () => void;
  saving: boolean;
  saveError: string | null;
  mobile?: boolean;
}) {
  const inputClass = mobile
    ? 'mobile-input w-full'
    : 'h-9 rounded-md border border-border bg-bg-subtle px-3 text-sm outline-none focus:border-accent';
  const addressClass = mobile
    ? 'mobile-input w-full font-mono text-sm'
    : 'h-9 rounded-md border border-border bg-bg-subtle px-3 text-xs outline-none focus:border-accent';
  const textareaClass = mobile
    ? 'mobile-textarea w-full'
    : 'rounded-md border border-border bg-bg-subtle px-3 py-2 text-sm outline-none focus:border-accent';
  const selectClass = mobile
    ? 'mobile-select w-full'
    : 'h-9 rounded-md border border-border bg-bg-subtle px-2 text-sm outline-none focus:border-accent';

  return (
    <div className="flex flex-col gap-3 text-sm">
      <div
        className={cn('grid gap-3', mobile ? 'grid-cols-1' : 'grid-cols-1 gap-2 sm:grid-cols-2')}
      >
        <div className="flex flex-col gap-1.5">
          <label className="text-xs font-medium text-fg-muted">Label</label>
          <input
            value={draft.label}
            onChange={(e) => onChange({ ...draft, label: e.target.value })}
            className={inputClass}
            placeholder="e.g. Exchange deposit"
          />
        </div>
        <div className="flex flex-col gap-1.5">
          <label className="text-xs font-medium text-fg-muted">Type</label>
          <select
            value={draft.category}
            onChange={(e) =>
              onChange({
                ...draft,
                category: e.target.value as AddressBookCategory,
              })
            }
            className={selectClass}
          >
            <option value="send">Send to</option>
            <option value="receive">Receive at</option>
          </select>
        </div>
      </div>
      <div className="flex flex-col gap-1.5">
        <label className="text-xs font-medium text-fg-muted">Address</label>
        <input
          value={draft.address}
          onChange={(e) => onChange({ ...draft, address: e.target.value })}
          spellCheck={false}
          autoCapitalize="off"
          autoCorrect="off"
          className={addressClass}
          placeholder="VTDns…"
        />
      </div>
      <div className="flex flex-col gap-1.5">
        <label className="text-xs font-medium text-fg-muted">Notes (optional)</label>
        <textarea
          value={draft.notes}
          onChange={(e) => onChange({ ...draft, notes: e.target.value })}
          rows={mobile ? 3 : 2}
          className={textareaClass}
        />
      </div>
      {saveError && <div className="text-xs text-danger">{saveError}</div>}
      <div className={cn('flex gap-2', mobile ? 'flex-col-reverse pt-1' : 'justify-end')}>
        <Button
          size={mobile ? 'md' : 'sm'}
          variant="ghost"
          className={mobile ? 'h-11 w-full rounded-xl' : undefined}
          onClick={onCancel}
          disabled={saving}
        >
          <X className="h-4 w-4" /> Cancel
        </Button>
        <Button
          size={mobile ? 'md' : 'sm'}
          className={mobile ? 'h-11 w-full rounded-xl' : undefined}
          onClick={onSave}
          disabled={!draft.address.trim() || saving}
        >
          <Save className="h-4 w-4" /> {saving ? 'Saving…' : 'Save'}
        </Button>
      </div>
    </div>
  );
}

function EntryRow({
  entry,
  onEdit,
  onDelete,
  mobile = false,
}: {
  entry: AddressBookEntry;
  onEdit: () => void;
  onDelete: () => void;
  mobile?: boolean;
}) {
  return (
    <li
      className={cn(
        mobile
          ? 'mobile-list-card flex items-start justify-between gap-3'
          : 'flex items-start justify-between gap-3 rounded-md border border-border bg-bg-subtle/40 px-3 py-2.5'
      )}
    >
      <div className="min-w-0 flex-1">
        <div className="flex flex-wrap items-center gap-2">
          <span className="text-sm font-semibold text-fg">{entry.label || '(no label)'}</span>
          <Badge tone="neutral">{entry.category}</Badge>
        </div>
        <div
          className={cn(
            'mt-1 break-all text-fg-muted',
            mobile ? 'text-xs leading-relaxed' : 'text-[11px]'
          )}
        >
          {entry.address}
        </div>
        {entry.notes && (
          <div className="mt-1.5 text-xs leading-relaxed text-fg-subtle">{entry.notes}</div>
        )}
      </div>
      <div className="flex shrink-0 gap-1">
        <Button
          size="sm"
          variant="ghost"
          className={mobile ? 'h-10 w-10 rounded-xl' : undefined}
          onClick={onEdit}
          aria-label="Edit"
        >
          <Pencil className="h-4 w-4" />
        </Button>
        <Button
          size="sm"
          variant="ghost"
          className={mobile ? 'h-10 w-10 rounded-xl' : undefined}
          onClick={onDelete}
          aria-label="Delete"
        >
          <Trash2 className="h-4 w-4" />
        </Button>
      </div>
    </li>
  );
}
