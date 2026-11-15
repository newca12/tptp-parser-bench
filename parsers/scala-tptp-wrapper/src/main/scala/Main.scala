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
    if (args.length == 4 && args(0) == "--bench") {
      val warmup = args(1).toInt
      val iters = args(2).toInt
      benchFile(warmup, iters, args(3))
      return
    }

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

  def benchFile(warmup: Int, iters: Int, path: String): Unit = {
    val content = try {
      java.nio.file.Files.readString(java.nio.file.Paths.get(path), java.nio.charset.StandardCharsets.UTF_8)
    } catch {
      case e: Exception =>
        System.err.println(s"error reading $path: ${e.getMessage}")
        System.exit(2)
        return
    }

    for (_ <- 0 until warmup) {
      val source = Source.fromString(content)
      try {
        TPTPParser.problem(source)
      } catch {
        case e: Exception =>
          System.err.println(s"error parsing $path: ${e.getMessage}")
          System.exit(1)
      } finally {
        source.close()
      }
    }

    val times = new Array[Long](iters)
    for (i <- 0 until iters) {
      val source = Source.fromString(content)
      val t0 = System.nanoTime()
      try {
        TPTPParser.problem(source)
      } catch {
        case e: Exception =>
          System.err.println(s"error parsing $path: ${e.getMessage}")
          System.exit(1)
      } finally {
        source.close()
      }
      times(i) = System.nanoTime() - t0
    }

    println(s"BENCH_RESULTS_NS: [${times.mkString(",")}]")
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
