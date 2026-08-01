import {
  type HistoryEntry,
  addHistoryEntry,
  clearHistory,
  getHistory,
  removeHistoryEntry,
} from '@/lib/history';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

export type { HistoryEntry };

export function useHistory() {
  const queryClient = useQueryClient();

  const query = useQuery({
    queryKey: ['history'],
    queryFn: getHistory,
    staleTime: 0,
  });

  const addEntry = useMutation({
    mutationFn: (entry: HistoryEntry) => addHistoryEntry(entry),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['history'] });
    },
  });

  const removeEntry = useMutation({
    mutationFn: (jobId: string) => removeHistoryEntry(jobId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['history'] });
    },
  });

  const clearAll = useMutation({
    mutationFn: () => clearHistory(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['history'] });
    },
  });

  return {
    entries: query.data ?? [],
    isLoading: query.isLoading,
    addEntry: addEntry.mutate,
    removeEntry: removeEntry.mutate,
    clearAll: clearAll.mutate,
  };
}
