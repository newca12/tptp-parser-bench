package tptp;

import org.antlr.v4.runtime.*;

/**
 * CLI wrapper for the ANTLR4-generated TPTP parser (Java target).
 *
 * Usage: java -jar java-antlr-wrapper.jar &lt;file.p&gt;
 *        java -jar java-antlr-wrapper.jar --check &lt;file.p&gt;
 *
 * Parses the given TPTP problem file and exits:
 *   0 on success
 *   1 on parse error
 *   2 on file/usage error
 *
 * Produces no output on success (designed for timing).
 *
 * In --check mode, outputs "OK formulas=N includes=M" to stdout on success.
 */
public class Main {
    public static void main(String[] args) {
        if (args.length == 2 && args[0].equals("--check")) {
            checkFile(args[1]);
            return;
        }

        if (args.length != 1) {
            System.err.println("usage: java-antlr-wrapper <file.p>");
            System.exit(2);
        }

        String path = args[0];

        try {
            CharStream input = CharStreams.fromFileName(path);
            TPTPLexer lexer = new TPTPLexer(input);
            CommonTokenStream tokens = new CommonTokenStream(lexer);
            TPTPParser parser = new TPTPParser(tokens);

            // Collect errors instead of printing to stderr
            parser.removeErrorListeners();
            lexer.removeErrorListeners();

            final boolean[] hasError = {false};
            BaseErrorListener errorListener = new BaseErrorListener() {
                @Override
                public void syntaxError(Recognizer<?, ?> recognizer,
                                        Object offendingSymbol,
                                        int line, int charPositionInLine,
                                        String msg,
                                        RecognitionException e) {
                    hasError[0] = true;
                }
            };
            parser.addErrorListener(errorListener);
            lexer.addErrorListener(errorListener);

            parser.tptp_file();

            if (hasError[0]) {
                System.err.println("parse error in " + path);
                System.exit(1);
            }
        } catch (Exception e) {
            System.err.println("error reading " + path + ": " + e.getMessage());
            System.exit(2);
        }
    }

    static void checkFile(String path) {
        try {
            CharStream input = CharStreams.fromFileName(path);
            TPTPLexer lexer = new TPTPLexer(input);
            CommonTokenStream tokens = new CommonTokenStream(lexer);
            TPTPParser parser = new TPTPParser(tokens);

            // Collect errors instead of printing to stderr
            parser.removeErrorListeners();
            lexer.removeErrorListeners();

            final boolean[] hasError = {false};
            BaseErrorListener errorListener = new BaseErrorListener() {
                @Override
                public void syntaxError(Recognizer<?, ?> recognizer,
                                        Object offendingSymbol,
                                        int line, int charPositionInLine,
                                        String msg,
                                        RecognitionException e) {
                    hasError[0] = true;
                }
            };
            parser.addErrorListener(errorListener);
            lexer.addErrorListener(errorListener);

            TPTPParser.Tptp_fileContext ctx = parser.tptp_file();

            if (hasError[0]) {
                System.err.println("parse error in " + path);
                System.exit(1);
            }

            // Count formulas and includes from parse tree
            int formulaCount = 0;
            int includeCount = 0;
            for (TPTPParser.Tptp_inputContext inputCtx : ctx.tptp_input()) {
                if (inputCtx.annotated_formula() != null) {
                    formulaCount++;
                } else if (inputCtx.include() != null) {
                    includeCount++;
                }
            }

            System.out.println("OK formulas=" + formulaCount + " includes=" + includeCount);
        } catch (Exception e) {
            System.err.println("error reading " + path + ": " + e.getMessage());
            System.exit(2);
        }
    }
}
