@echo off
cd %*
if "%ENM_SIFT_METHOD%" == "recursive" (
  enm switch --caps-lock-when-needed
) else (
  if exist .nvmrc (
    enm switch --caps-lock-when-needed
  ) else (
    if exist .node-version (
      enm switch --caps-lock-when-needed
    )
  )
)
@echo on
