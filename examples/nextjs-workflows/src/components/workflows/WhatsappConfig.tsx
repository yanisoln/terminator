"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { toast } from "sonner";
import { Badge } from "@/components/ui/badge";

export default function WhatsappConfig() {
  const [isLoading, setIsLoading] = useState(false);

  const handleTriggerExport = async () => {
    toast.info("this feature is coming soon!");
    return;

    // setIsLoading(true);
    // toast.info("attempting to trigger whatsapp export...");
    // try {
    //   const response = await fetch("/api/trigger-whatsapp-export", {
    //     method: "POST",
    //     // No body needed for this simple trigger
    //   });
    //   const result = await response.json();
    //   if (!response.ok) {
    //     throw new Error(result.message || "failed to trigger export.");
    //   }
    //   toast.success(result.message || "export process initiated successfully.");
    // } catch (error) {
    //   console.error("trigger export error:", error);
    //   toast.error(
    //     `error: ${
    //       error instanceof Error ? error.message : "an unknown error occurred."
    //     }`
    //   );
    // } finally {
    //   setIsLoading(false);
    // }
  };

  return (
    <div className="space-y-4">
       <Badge variant="outline">coming soon</Badge>
      <p className="text-sm text-muted-foreground">
        this workflow will automate exporting a whatsapp chat and processing its contents. ensure whatsapp desktop is running and the desired chat is open and active before triggering.
        {/* Original text commented out for now:
        ensure whatsapp desktop is running and the desired chat is open and
        active. clicking the button will attempt to automate the chat export
        process. */}
      </p>
      <Button onClick={handleTriggerExport} disabled={true /* Always disabled for now */}>
        {/* {isLoading
          ? "initiating export..."
          : "export active whatsapp chat (zip)"} */}
          export active whatsapp chat (coming soon)
      </Button>
    </div>
  );
}
