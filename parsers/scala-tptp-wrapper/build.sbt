name := "scala-tptp-wrapper"
version := "0.1.0"
scalaVersion := "3.9.0"

libraryDependencies += "io.github.leoprover" %% "scala-tptp-parser" % "1.7.5"

assembly / mainClass := Some("Main")
assembly / assemblyJarName := "scala-tptp-wrapper.jar"

assembly / assemblyMergeStrategy := {
  case PathList("META-INF", _*) => MergeStrategy.discard
  case _ => MergeStrategy.first
}
