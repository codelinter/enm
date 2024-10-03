@echo off
cd %*
if "%ENM_VERSION_FILE_STRATEGY%" == "recursive" (
  enm use --silent-if-unchanged
) else (
  if exist .nvmrc (
    enm use --silent-if-unchanged
  ) else (
    if exist .node-version (
      enm use --silent-if-unchanged
    )
  )
)
@echo on
