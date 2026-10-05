# AP29: Bounded derivation

Status: rejected and closed by decision Q22; removed from the active plan. This
package has no work packages. Completion is tracked in the
[central README](../README.md).

## Decision (Q22)

Records and enums already have structural equality, the package's main
proposed use case. Do not add derivation syntax or machinery. If concrete needs
for debug rendering or ordering arise, address them with ordinary
standard-library functions.
