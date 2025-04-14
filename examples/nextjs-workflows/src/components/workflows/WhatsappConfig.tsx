"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { toast } from "sonner";

export default function WhatsappConfig() {
  const [isLoading, setIsLoading] = useState(false);

  const handleTriggerExport = async () => {
    setIsLoading(true);
    toast.info("attempting to trigger whatsapp export...");

    try {
      const response = await fetch("/api/trigger-whatsapp-export", {
        method: "POST",
        // No body needed for this simple trigger
      });

      const result = await response.json();

      if (!response.ok) {
        throw new Error(result.message || "failed to trigger export.");
      }

      toast.success(result.message || "export process initiated successfully.");
      // Note: This doesn't confirm the export *completed* successfully,
      // only that the API call to start the automation was received.
    } catch (error) {
      console.error("trigger export error:", error);
      toast.error(
        `error: ${
          error instanceof Error ? error.message : "an unknown error occurred."
        }`
      );
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="space-y-4">
      <p className="text-sm text-muted-foreground">
        ensure whatsapp desktop is running and the desired chat is open and
        active. clicking the button will attempt to automate the chat export
        process.
      </p>
      <Button onClick={handleTriggerExport} disabled={isLoading}>
        {isLoading
          ? "initiating export..."
          : "export active whatsapp chat (zip)"}
      </Button>
    </div>
  );
}
