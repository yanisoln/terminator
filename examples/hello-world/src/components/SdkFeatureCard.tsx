import React from "react";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

interface SdkFeatureCardProps {
  title: string;
  description: string;
  codeSnippet: string;
  children: React.ReactNode; // For interactive elements like buttons
}

const SdkFeatureCard: React.FC<SdkFeatureCardProps> = ({
  title,
  description,
  codeSnippet,
  children,
}) => {
  return (
    <Card>
      <CardHeader>
        <CardTitle>{title}</CardTitle>
        <CardDescription>{description}</CardDescription>
      </CardHeader>
      <CardContent>
        <pre className="mb-4 overflow-x-auto rounded-md bg-gray-100 p-3 dark:bg-gray-800">
          <code className="text-sm font-mono">{codeSnippet}</code>
        </pre>
        <div>{children}</div>
      </CardContent>
    </Card>
  );
};

export default SdkFeatureCard;
