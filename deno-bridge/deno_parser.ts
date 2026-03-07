// deno-bridge/deno_parser.ts
// TypeScript Parser Bridge using Deno and TypeScript Compiler API
// Reads TypeScript source from stdin, parses it, and writes serialized AST to stdout

import * as ts from "https://esm.sh/typescript@5.3.3";
import { encode } from "https://esm.sh/@msgpack/msgpack@2.8.0";

/**
 * Serializes a TypeScript AST node to a structured format compatible with Rust
 */
function serializeNode(node: ts.Node): any {
  const kind = ts.SyntaxKind[node.kind];

  const result: any = {
    kind: kind,
    pos: node.pos,
    end: node.end,
  };

  // Serialize specific properties based on node type
  switch (node.kind) {
    // Source File
    case ts.SyntaxKind.SourceFile: {
      const sf = node as ts.SourceFile;
      result.statements = sf.statements.map(serializeNode);
      result.fileName = sf.fileName;
      break;
    }

    // Function Declaration
    case ts.SyntaxKind.FunctionDeclaration: {
      const fn = node as ts.FunctionDeclaration;
      result.name = fn.name?.text;
      result.parameters = fn.parameters.map(serializeNode);
      result.returnType = fn.type ? serializeNode(fn.type) : undefined;
      result.body = fn.body ? serializeNode(fn.body) : undefined;
      break;
    }

    // Parameter
    case ts.SyntaxKind.Parameter: {
      const param = node as ts.ParameterDeclaration;
      result.name = param.name.getText();
      result.type = param.type ? serializeNode(param.type) : undefined;
      result.defaultValue = param.initializer ? serializeNode(param.initializer) : undefined;
      result.isRest = !!param.dotDotDotToken;
      break;
    }

    // Block Statement
    case ts.SyntaxKind.Block: {
      const block = node as ts.Block;
      result.statements = block.statements.map(serializeNode);
      break;
    }

    // Return Statement
    case ts.SyntaxKind.ReturnStatement: {
      const ret = node as ts.ReturnStatement;
      result.value = ret.expression ? serializeNode(ret.expression) : undefined;
      break;
    }

    // Variable Statement
    case ts.SyntaxKind.VariableStatement: {
      const stmt = node as ts.VariableStatement;
      result.declarations = stmt.declarationList.declarations.map(serializeNode);
      result.kind = stmt.declarationList.flags & ts.NodeFlags.Let ? "Let" :
                   stmt.declarationList.flags & ts.NodeFlags.Const ? "Const" : "Var";
      break;
    }

    // Variable Declaration
    case ts.SyntaxKind.VariableDeclaration: {
      const decl = node as ts.VariableDeclaration;
      result.name = decl.name.getText();
      result.initializer = decl.initializer ? serializeNode(decl.initializer) : undefined;
      result.type = decl.type ? serializeNode(decl.type) : undefined;
      break;
    }

    // If Statement
    case ts.SyntaxKind.IfStatement: {
      const ifStmt = node as ts.IfStatement;
      result.condition = serializeNode(ifStmt.expression);
      result.thenStatement = serializeNode(ifStmt.thenStatement);
      result.elseStatement = ifStmt.elseStatement ? serializeNode(ifStmt.elseStatement) : undefined;
      break;
    }

    // While Statement
    case ts.SyntaxKind.WhileStatement: {
      const whileStmt = node as ts.WhileStatement;
      result.condition = serializeNode(whileStmt.expression);
      result.body = serializeNode(whileStmt.statement);
      break;
    }

    // Expression Statement
    case ts.SyntaxKind.ExpressionStatement: {
      const exprStmt = node as ts.ExpressionStatement;
      result.expression = serializeNode(exprStmt.expression);
      break;
    }

    // Binary Expression
    case ts.SyntaxKind.BinaryExpression: {
      const binExpr = node as ts.BinaryExpression;
      result.operator = ts.SyntaxKind[binExpr.operatorToken.kind];
      result.left = serializeNode(binExpr.left);
      result.right = serializeNode(binExpr.right);
      break;
    }

    // Identifier
    case ts.SyntaxKind.Identifier: {
      const id = node as ts.Identifier;
      result.name = id.text;
      break;
    }

    // String Literal
    case ts.SyntaxKind.StringLiteral: {
      const lit = node as ts.StringLiteral;
      result.value = lit.text;
      result.literalType = "String";
      break;
    }

    // Numeric Literal
    case ts.SyntaxKind.NumericLiteral: {
      const lit = node as ts.NumericLiteral;
      result.value = parseFloat(lit.text);
      result.literalType = "Number";
      break;
    }

    // Boolean Literal
    case ts.SyntaxKind.TrueKeyword:
    case ts.SyntaxKind.FalseKeyword: {
      result.value = node.kind === ts.SyntaxKind.TrueKeyword;
      result.literalType = "Boolean";
      break;
    }

    // Null Literal
    case ts.SyntaxKind.NullKeyword: {
      result.value = null;
      result.literalType = "Null";
      break;
    }

    // Call Expression
    case ts.SyntaxKind.CallExpression: {
      const call = node as ts.CallExpression;
      result.callee = serializeNode(call.expression);
      result.arguments = call.arguments.map(serializeNode);
      break;
    }

    // Template Literal
    case ts.SyntaxKind.TemplateLiteral: {
      const tmpl = node as ts.TemplateLiteral;
      result.parts = [];

      // Add head
      result.parts.push({
        type: "Static",
        value: tmpl.head.text,
      });

      // Add template spans
      for (const span of tmpl.templateSpans) {
        result.parts.push({
          type: "Expression",
          expression: serializeNode(span.expression),
        });
        result.parts.push({
          type: "Static",
          value: span.literal.text,
        });
      }
      break;
    }

    // Type Reference
    case ts.SyntaxKind.TypeReference: {
      const typeRef = node as ts.TypeReferenceNode;
      result.typeName = typeRef.typeName.getText();
      result.typeArguments = typeRef.typeArguments?.map(serializeNode);
      break;
    }

    // Array Type
    case ts.SyntaxKind.ArrayType: {
      const arrayType = node as ts.ArrayTypeNode;
      result.elementType = serializeNode(arrayType.elementType);
      break;
    }

    // Union Type
    case ts.SyntaxKind.UnionType: {
      const unionType = node as ts.UnionTypeNode;
      result.types = unionType.types.map(serializeNode);
      break;
    }

    // Add more node types as needed...

    default:
      // For unhandled node types, store minimal info
      result.text = node.getText();
      break;
  }

  return result;
}

/**
 * Reads all input from stdin
 */
async function readStdin(): Promise<string> {
  const chunks: Uint8Array[] = [];
  for await (const chunk of Deno.stdin.readable) {
    chunks.push(chunk);
  }
  return new TextDecoder().decode(Uint8Array.from(chunks.flat()));
}

/**
 * Main function
 */
async function main() {
  try {
    // Read source from stdin
    const source = await readStdin();

    // Parse TypeScript
    const sourceFile = ts.createSourceFile(
      "input.ts",
      source,
      ts.ScriptTarget.Latest,
      true // Set this to true to parse TypeScript
    );

    // Serialize AST
    const ast = serializeNode(sourceFile);

    // Prepare response
    const response = {
      success: true,
      ast: ast,
      errors: [],
    };

    // Encode as MessagePack and write to stdout
    const encoded = encode(response);
    await Deno.stdout.write(encoded);

  } catch (error) {
    // Handle errors
    const errorResponse = {
      success: false,
      error: error instanceof Error ? error.message : String(error),
    };
    const encoded = encode(errorResponse);
    await Deno.stdout.write(encoded);
    Deno.exit(1);
  }
}

// Run main
main().catch((error) => {
  console.error("Fatal error:", error);
  Deno.exit(1);
});
