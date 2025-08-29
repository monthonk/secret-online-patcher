# Secret Online Patcher

A CLI tool for managing application versions and tracking changes through hash codes.

## Usage

**List all applications:**
```bash
secret-online-patcher list
```

**Add a new application:**
```bash
secret-online-patcher add-app --app-name <NAME> --app-version <VERSION> --app-path <PATH>
```

### Examples

```bash
# List applications
secret-online-patcher list

# Add an application
secret-online-patcher add-app --app-name "MyApp" --app-version "1.0.0" --app-path "/path/to/app"
```

The application stores data in `resources/app_data.db` which is automatically created on first run.
