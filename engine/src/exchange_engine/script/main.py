import uvicorn
from api import create_app

app = create_app()

def start_server():
    """启动FastAPI服务器"""
    uvicorn.run(
        app,
        host="0.0.0.0",
        port=8000,
        reload=False
    )

if __name__ == "__main__":
    start_server() 