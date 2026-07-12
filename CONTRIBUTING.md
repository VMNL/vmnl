# Git Commit Guidelines

## Adding a Feature

```
git checkout -b [feature/hotfix]/[featureName] [Branch you want you feature in]
```

Creates the feature branch off your branch. Do your work and then


```
git add [files to be commited]
git commit -m "< :gitmoji: >[scope/feature, hotfix, ...] ..."
```

Now merge your changes to your branch without a fast-forward (to avoid your branch to be squished)

```
git checkout [Branch you want you feature in]
git merge --no-ff [feature/hotfix]/[featureName]
```

Finally, push the changes to the server

```
git push origin [Branch you want you feature in]
```

#### [Source](https://stackoverflow.com/questions/4470523/create-a-branch-in-git-from-another-branch)

## Committing

```
< :gitmoji: >[scope/feature] Title
Description
```

### Exemple :
```bash
git add main.cpp
git commit -m "Title" -m "Descrption"
git push
```

`:gitmoji:` -> Describe the type of change
[Get the list](https://gitmoji.dev/)

`scope` -> Branch of the project
- `server` -> server branch
- `engine` -> engine Branch
- `client` -> client branch

`feature` -> Feature name (The branch name usually)

`Title` -> Title of the commit, short and imperative <= 50 characters

`Description` (Optional) Explain what and why

- Add `Closes #XXX (X is a number of closes)` to close an issue/PR automatically
- Add `BREAKING CHANGE:` if it breaks backward compatibility (e.g. renaming a public function or class)
