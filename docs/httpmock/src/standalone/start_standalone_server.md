# Using the Docker Image 

## Using the Official Docker Image on Docker Hub
The Dockerfile available at https://hub.docker.com/r/alexliesenfeld/httpmock provides a convenient way to run the `httpmock`library as a standalone server using Docker. By using this Docker image, you can easily set up and deploy the mock server in your development or testing environment.

To use the `httpmock`Docker image, follow these steps:

Pull the Docker image: Use the docker pull command to download the `httpmock`image from the Docker Hub. Run the following command in your terminal:

```bash
docker pull alexliesenfeld/httpmock
```
Run the Docker container: Once the image is downloaded, run a Docker container using the `httpmock`image. This will start the standalone mock server. Use the docker run command, specifying any necessary port mappings or environment variables required by your specific use case. For example:

```sh
docker run -p 5000:5000 -d alexliesenfeld/httpmock
```
In this example, the -p flag maps port `5000` from the container to the host machine, allowing you to access the mock server 
at http://localhost:5000. Adjust the port mapping as needed for your environment.

Interact with the mock server: With the Docker container running, you can now interact with the standalone mock server. 
Use the appropriate HTTP client library or tool to send HTTP requests to the mock server, targeting the specified port (e.g., 5000) and any specific routes or endpoints defined in your mocks.

For example, using cURL, you can send an HTTP GET request to the mock server running on `localhost:5000`:

```bash
curl http://localhost:5000/some-endpoint
```

The mock server will respond based on the defined mocks and expectations, providing the expected HTTP responses.

By leveraging the `httpmock`Docker image, you can easily deploy the standalone mock server in any environment with Docker support. This approach simplifies the setup process and enables you to simulate and control HTTP responses for your applications during development, testing, or any scenario where a mock server is required.

## Using the Dockerfile From the GitHub Repository
There is also a Dockerfile that you can use to build your own image if you like.

To use the local Dockerfile and build your own image for the `httpmock` standalone server, follow these steps:

Ensure you have Docker installed on your machine. You can download and install Docker from the official Docker website (https://www.docker.com/products/docker-desktop).

Open a terminal or command prompt and navigate to the main directory of the `httpmock`repository where the Dockerfile is located.

Run the following command to build the Docker image:

```sh
docker build -t my-httpmock-image .
```

This command instructs Docker to build an image based on the Dockerfile in the current directory (.) and tags the image with the name `my-httpmock-image`. You can replace `my-httpmock-image` with any desired name for your image.

Docker will start building the image, pulling any required dependencies and executing the instructions specified in the Dockerfile. The build process may take a few minutes to complete.

Once the build is finished, you can verify that your image has been created by running the command:

```sh
docker images
```
You should see your newly built image, `my-httpmock-image`, listed among the available Docker images.

Now you have successfully built your own Docker image for the `httpmock`standalone server. You can proceed to create and run containers from this image, configuring them according to your specific requirements and environment.