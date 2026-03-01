import leo.modules.input.TPTPParser

import scala.io.Source
import scala.util.{Failure, Success, Try}

/** CLI wrapper for scala-tptp-parser.
  *
  * Usage: java -jar scala-tptp-wrapper.jar <file.p>
  *        java -jar scala-tptp-wrapper.jar --check <file.p>
  *
  * Parses the given TPTP problem file and exits:
  *   0 on success
  *   1 on parse error
  *   2 on file/usage error
  *
  * Produces no output on success (designed for timing).
  *
  * In --check mode, outputs "OK formulas=N includes=M" to stdout on success,
  * or "ROUNDTRIP_FAIL formulas=N includes=M error=..." on roundtrip failure.
  */
object Main {
  def main(args: Array[String]): Unit = {
    if (args.length == 2 && args(0) == "--check") {
      checkFile(args(1))
      return
    }

    if (args.length != 1) {
      System.err.println("usage: scala-tptp-wrapper <file.p>")
      System.exit(2)
    }

    val path = args(0)

    val source = Try(Source.fromFile(path, "UTF-8")) match {
      case Success(s) => s
      case Failure(e) =>
        System.err.println(s"error reading $path: ${e.getMessage}")
        System.exit(2)
        throw e // unreachable, satisfies compiler
    }

    try {
      TPTPParser.problem(source)
    } catch {
      case e: TPTPParser.TPTPParseException =>
        System.err.println(s"parse error in $path: ${e.getMessage}")
        System.exit(1)
      case e: Exception =>
        System.err.println(s"error parsing $path: ${e.getMessage}")
        System.exit(1)
    } finally {
      source.close()
    }
  }

  def checkFile(path: String): Unit = {
    val source = Try(Source.fromFile(path, "UTF-8")) match {
      case Success(s) => s
      case Failure(e) =>
        System.err.println(s"error reading $path: ${e.getMessage}")
        System.exit(2)
        throw e
    }

    try {
      val problem = TPTPParser.problem(source)
      val formulaCount = problem.formulas.size
      val includeCount = problem.includes.size

      // Roundtrip check: parse -> pretty -> reparse -> compare
      val roundtripError = problem.formulas.view.flatMap { formula =>
        val pretty = formula.pretty
        Try(TPTPParser.annotated(pretty)) match {
          case Success(reparsed) =>
            val reparsedPretty = reparsed.pretty
            if (pretty != reparsedPretty)
              Some(s"Round-trip mismatch for '${formula.name}':\n  Original:  $pretty\n  Reparsed:  $reparsedPretty")
            else None
          case Failure(e) =>
            Some(s"Failed to reparse '${formula.name}': ${e.getMessage}\n  Pretty: $pretty")
        }
      }.headOption

      roundtripError match {
        case Some(error) =>
          println(s"ROUNDTRIP_FAIL formulas=$formulaCount includes=$includeCount error=$error")
          System.exit(1)
        case None =>
          println(s"OK formulas=$formulaCount includes=$includeCount")
      }
    } catch {
      case e: TPTPParser.TPTPParseException =>
        System.err.println(s"parse error in $path: ${e.getMessage}")
        System.exit(1)
      case e: Exception =>
        System.err.println(s"error parsing $path: ${e.getMessage}")
        System.exit(1)
    } finally {
      source.close()
    }
  }
}
