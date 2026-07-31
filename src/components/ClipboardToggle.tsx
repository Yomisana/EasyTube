import { useState, useCallback } from "react";
import { useTranslations } from "use-intl";
import { Switch } from "@/components/ui/switch";

interface ClipboardToggleProps {
  onToggle?: (enabled: boolean) => void;
}

export function ClipboardToggle({ onToggle }: ClipboardToggleProps) {
  const t = useTranslations("clipboard");
  const [enabled, setEnabled] = useState(false);

  const handleChange = useCallback(
    (checked: boolean) => {
      setEnabled(checked);
      onToggle?.(checked);
    },
    [onToggle],
  );

  return (
    <div className="flex items-center gap-3">
      <Switch
        checked={enabled}
        onCheckedChange={handleChange}
        id="clipboard-monitor"
      />
      <label
        htmlFor="clipboard-monitor"
        className="text-sm font-medium leading-none cursor-pointer select-none"
      >
        {t("toggle")}
      </label>
    </div>
  );
}
