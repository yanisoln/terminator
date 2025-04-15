"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { toast } from "sonner";
import { Badge } from "@/components/ui/badge";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { z } from 'zod';

// Define the expected structure for validation (matches backend)
const spreadsheetSchema = z.object({
  columns: z.array(z.string()),
  rows: z.array(z.array(z.string())),
});

// Type for the spreadsheet data state
type SpreadsheetData = z.infer<typeof spreadsheetSchema> | null;

export default function PdfToSpreadsheetConfig() {
  const [isLoading, setIsLoading] = useState(false);
  const [apiKey, setApiKey] = useState("");
  const [spreadsheetData, setSpreadsheetData] = useState<SpreadsheetData>(null);

  const handleTriggerProcessing = async () => {
    if (!apiKey) {
      toast.error("Please enter your Gemini API Key.");
      return;
    }

    setIsLoading(true);
    setSpreadsheetData(null);
    toast.info("Attempting to process PDF...");

    try {
      const response = await fetch("/api/process-pdf", {
        method: "POST",
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ apiKey: apiKey })
      });

      const result = await response.json();

      if (!response.ok) {
        throw new Error(result.message || "Failed to trigger PDF processing.");
      }

      // Validate and set the spreadsheet data
      try {
        const validatedData = spreadsheetSchema.parse(result.spreadsheetData);
        setSpreadsheetData(validatedData);
        toast.success(result.message || "PDF processing workflow completed successfully.");
      } catch (validationError) {
        console.error("Frontend validation error:", validationError);
        setSpreadsheetData(null);
        toast.error("Received invalid spreadsheet data format from backend.");
      }

    } catch (error) {
      console.error("Trigger PDF processing error:", error);
      setSpreadsheetData(null);
      toast.error(
        `Error: ${error instanceof Error ? error.message : "An unknown error occurred."}`
      );
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-medium mb-2">PDF to Spreadsheet Workflow</h3>
        <p className="text-sm text-muted-foreground">
          This workflow downloads a PDF, opens it, uses AI to extract structured data,
          converts it to a spreadsheet format, and then focuses back on this browser tab.
          It requires your Gemini API Key for the AI processing step.
        </p>
      </div>

      <div className="space-y-2">
        <Label htmlFor="apiKey">Gemini API Key</Label>
        <Input
          id="apiKey"
          type="password"
          placeholder="Enter your Gemini API Key"
          value={apiKey}
          onChange={(e) => setApiKey(e.target.value)}
          disabled={isLoading}
        />
        <p className="text-xs text-muted-foreground">
          Your API key is sent directly to the backend for processing and is not stored long-term.
        </p>
      </div>

      <div className="flex items-center space-x-4">
        <Button onClick={handleTriggerProcessing} disabled={isLoading || !apiKey}>
          {isLoading ? "Processing..." : "Process Example PDF to Spreadsheet"}
        </Button>
        <Badge variant="secondary">Demo</Badge>
      </div>

      {/* Display Spreadsheet Data */}
      {spreadsheetData && (
        <div className="mt-6 p-4 border rounded-md bg-muted/50">
            <h4 className="text-md font-medium mb-3">Extracted Spreadsheet Data</h4>
            {spreadsheetData.columns.length > 0 && spreadsheetData.rows.length > 0 ? (
            <Table>
                <TableHeader>
                <TableRow>
                    {spreadsheetData.columns.map((header, index) => (
                    <TableHead key={index}>{header}</TableHead>
                    ))}
                </TableRow>
                </TableHeader>
                <TableBody>
                {spreadsheetData.rows.map((row, rowIndex) => (
                    <TableRow key={rowIndex}>
                    {row.map((cell, cellIndex) => (
                        <TableCell key={cellIndex}>{cell}</TableCell>
                    ))}
                    </TableRow>
                ))}
                </TableBody>
            </Table>
            ) : (
                 <p className="text-sm text-muted-foreground">No data extracted or data format is empty.</p>
            )}
        </div>
        )}

      <p className="text-xs text-muted-foreground pt-2 border-t border-border">
        Note: This currently uses a hardcoded example PDF URL in the backend.
        The UI element locators in the backend might need adjustment for your specific PDF reader.
      </p>
    </div>
  );
} 