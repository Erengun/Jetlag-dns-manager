function Resolve-Symlinks {
    [CmdletBinding()]
    [OutputType([string])]
    param(
        [Parameter(Position = 0, Mandatory)]
        [string] $Path
    )

    [string] $separator = '/'
    [string] $normalizedPath = $Path.Replace('\', '/').TrimEnd('/')
    [string[]] $parts = $normalizedPath.Split($separator)

    [string] $realPath = ''
    if ($normalizedPath.StartsWith('//')) {
        $realPath = '//'
    } elseif ($normalizedPath.StartsWith('/')) {
        $realPath = '/'
    }

    foreach ($part in $parts) {
        if ([string]::IsNullOrEmpty($part)) {
            continue
        }

        if ($realPath -and !$realPath.EndsWith($separator)) {
            $realPath += $separator
        }

        $realPath += $part.Replace('\', '/')

        # The slash is important when using Get-Item on Drive letters in pwsh.
        if (-not($realPath.Contains($separator)) -and $realPath.EndsWith(':')) {
            $realPath += '/'
        }

        $item = Get-Item $realPath
        if ($item.LinkTarget) {
            $linkTarget = $item.LinkTarget.Replace('\', '/')
            if ([System.IO.Path]::IsPathRooted($linkTarget)) {
                $realPath = $linkTarget
            } else {
                $parentDir = ($realPath -replace '/[^/]+$', '').TrimEnd('/')
                $realPath = "$parentDir/$linkTarget"
            }
        }
    }
    $realPath
}

$path = Resolve-Symlinks -Path $args[0]
Write-Output $path
