import Link from "next/link";
import { Button } from "@/components/ui/button";
import WhatsappConfig from "@/components/workflows/WhatsappConfig";
import PdfToSpreadsheetConfig from "@/components/workflows/PdfToSpreadsheetConfig";

interface WorkflowPageProps {
  params: Promise<{
    slug: string;
  }>;
}

// Simple component map - can be extended
const workflowComponents: Record<string, React.ComponentType> = {
  "whatsapp-to-spreadsheet": WhatsappConfig,
  "pdf-to-spreadsheet": PdfToSpreadsheetConfig,
};

export default async function WorkflowPage(props: WorkflowPageProps) {
  const params = await props.params;

  // In a real app, you might fetch workflow details based on the slug
  const workflowTitle = params.slug
    .split("-")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");

  const ConfigComponent = workflowComponents[params.slug];

  return (
    <main className="container mx-auto px-4 py-8">
      <div className="mb-6 flex items-center justify-between">
        <h1 className="text-2xl font-semibold tracking-tight">
          {workflowTitle}
        </h1>
        <Link href="/" passHref>
          <Button variant="outline">back to workflows</Button>
        </Link>
      </div>
      <div className="rounded-lg border bg-card text-card-foreground shadow-sm p-6">
        <h2 className="mb-4 text-lg font-medium">configuration</h2>
        {ConfigComponent ? (
          <ConfigComponent />
        ) : (
          <p className="text-sm text-muted-foreground">
            no specific configuration ui available for "{workflowTitle}".
          </p>
        )}
      </div>
    </main>
  );
}
