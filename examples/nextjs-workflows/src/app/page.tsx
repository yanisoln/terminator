import Link from "next/link";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";

interface Workflow {
  slug: string;
  title: string;
  description: string;
}

const workflows: Workflow[] = [
  {
    slug: "whatsapp-to-spreadsheet",
    title: "whatsapp to spreadsheet",
    description:
      "turn your whatsapp chat exports into structured spreadsheets.",
  },
  {
    slug: "pdf-to-spreadsheet",
    title: "pdf to spreadsheet",
    description:
      "turn your pdfs into structured spreadsheets.",
  },
  // add more workflows here
];

export default function HomePage() {
  return (
    <main className="container mx-auto px-4 py-8">
      <h1 className="mb-6 text-2xl font-semibold tracking-tight">
        available workflows
      </h1>
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
        {workflows.map((workflow) => {
          const isComingSoon = workflow.slug === "whatsapp-to-spreadsheet";
          return (
            <Card key={workflow.slug}>
              <CardHeader>
                <div className="flex items-center justify-between">
                  <CardTitle className="text-lg">{workflow.title}</CardTitle>
                  {isComingSoon && <Badge variant="outline">coming soon</Badge>}
                </div>
                <CardDescription>{workflow.description}</CardDescription>
              </CardHeader>
              <CardFooter>
                {isComingSoon ? (
                   <Button variant="outline" size="sm" disabled>
                     configure (soon)
                   </Button>
                ) : (
                  <Link href={`/workflow/${workflow.slug}`} passHref>
                    <Button variant="outline" size="sm">
                      configure
                    </Button>
                  </Link>
                )}
              </CardFooter>
            </Card>
          );
        })}
      </div>
    </main>
  );
}
