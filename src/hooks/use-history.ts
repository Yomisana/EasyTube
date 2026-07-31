import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  getHistory,
  addHistoryEntry,
  removeHistoryEntry,
  clearHistory,
  type HistoryEntry,
} from "@/lib/history";

export function useHistory() {
  const queryClient = useQueryClient();

  const query = useQuery({
    queryKey: ["history"],
    queryFn: getHistory,
    staleTime: 0,
  });

  const addEntry = useMutation({
    mutationFn: (entry: HistoryEntry) => addHistoryEntry(entry),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["history"] });
    },
  });

  const removeEntry = useMutation({
    mutationFn: (jobId: string) => removeHistoryEntry(jobId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["history"] });
    },
  });

  const clearAll = useMutation({
    mutationFn: () => clearHistory(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["history"] });
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
