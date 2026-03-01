import leo.modules.input.TPTPParser

import scala.io.Source
import scala.util.{Failure, Success, Try}

/** CLI wrapper for scala-tptp-parser.
  *
  * Usage: java -jar scala-tptp-wrapper.jar <file.p>
  *
  * Parses the given TPTP problem file and exits:
  *   0 on success
  *   1 on parse error
  *   2 on file/usage error
  *
  * Produces no output on success (designed for timing).
  */
object Main {
  def main(args: Array[String]): Unit = {
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
}
