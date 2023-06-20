# Using YAML Files for Static Mock Definition
YAML files are commonly used to define static mock configurations in `httpmock` because they offer a simple and 
human-readable format. A significant advantage of using YAML files for static mock definitions is the ability to 
define mocks once and reuse them across different teams, projects, or instances. This promotes consistency 
and reduces duplication of efforts. Let's delve into this advantage further:

1. **Centralized mock definitions**: By keeping mock definitions in a static YAML file, you establish a central source of truth for your mocks. This allows different teams or projects within an organization to access and utilize the same mock configurations, ensuring consistent behavior across multiple instances.
2. **Time and effort savings**: Instead of redefining the same mocks repeatedly, teams can simply refer to the shared YAML file to obtain the desired mock configurations. This saves time and effort, as the mock definitions are already documented and readily available for use.
3. **Consistent testing environments**: When multiple teams or projects rely on the same mock configurations, it ensures that all instances are using consistent mocks during development, testing, and integration phases. This promotes better collaboration and avoids discrepancies in behavior between different environments.
4. **Version control and updates**: YAML files can be easily version-controlled using tools like Git. This enables tracking changes to the mock configurations, rolling back to previous versions if necessary, and applying updates across different instances. Teams can collaborate on refining and enhancing the mocks in the shared YAML file over time.

## How to use YAML files
To utilize static mock support in standalone httpmock, you have two options depending on how you are running the server:

### Using the Docker Image
If you are using the Docker image from the official `httpmock` repository or Docker Hub, you can follow these steps:

1. Create a directory on your local machine that contains all your mock specification files in YAML format.
2. Start the `httpmock` server using the Docker image and mount the directory with your mock specification files to the /mocks directory within the container.

For example, assuming your mock specification files are located in the /path/to/mocks directory, you can use the following command to start the server:

```bash
docker run -v /path/to/mocks:/mocks -p 5000:5000 alexliesenfeld/httpmock
```

This command mounts the local /path/to/mocks directory to the /mocks directory within the container, allowing `httpmock` to access and utilize the mock specification files.

### Building `httpmock` from source:

If you prefer to build `httpmock` from source and use the binary directly, you can specify the directory containing your mock specification files using the --static-mock-dir parameter.

For example, assuming your mock specification files are located in the /path/to/mocks directory, you can start the `httpmock` server as follows:

```bash
httpmock --expose --static-mock-dir=/path/to/mocks
```

By providing the `--static-mock-dir` parameter with the appropriate directory path, `httpmock` will load the mock specification files from that directory and make them available for use as static mocks.

Both approaches allow you to leverage YAML files for defining static mocks. By providing the server with the location of your mock specification files, `httpmock` will read and utilize those files to serve the defined mocks. This enables you to define mocks once in YAML format and easily reuse them across different instances, promoting consistency and efficiency in your API development and testing workflows.