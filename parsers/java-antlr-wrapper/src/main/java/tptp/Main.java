package tptp;

import org.antlr.v4.runtime.*;

/**
 * CLI wrapper for the ANTLR4-generated TPTP parser (Java target).
 *
 * Usage: java -jar java-antlr-wrapper.jar &lt;file.p&gt;
 *
 * Parses the given TPTP problem file and exits:
 *   0 on success
 *   1 on parse error
 *   2 on file/usage error
 *
 * Produces no output on success (designed for timing).
 */
public class Main {
    public static void main(String[] args) {
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
}
