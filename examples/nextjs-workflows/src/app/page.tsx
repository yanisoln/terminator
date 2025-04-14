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
  // add more workflows here
];

export default function HomePage() {
  return (
    <main className="container mx-auto px-4 py-8">
      <h1 className="mb-6 text-2xl font-semibold tracking-tight">
        available workflows
      </h1>
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
        {workflows.map((workflow) => (
          <Card key={workflow.slug}>
            <CardHeader>
              <CardTitle className="text-lg">{workflow.title}</CardTitle>
              <CardDescription>{workflow.description}</CardDescription>
            </CardHeader>
            <CardFooter>
              <Link href={`/workflow/${workflow.slug}`} passHref>
                <Button variant="outline" size="sm">
                  configure
                </Button>
              </Link>
            </CardFooter>
          </Card>
        ))}
      </div>
    </main>
  );
}
