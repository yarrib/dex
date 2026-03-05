"""Tool registry — auto-discovers tools in this package."""

from langchain_core.tools import BaseTool

{% if include_vector_search %}
from {{ project_name }}.tools.retriever import build_retriever_tool
{% endif %}


def get_tools() -> list[BaseTool]:
    tools: list[BaseTool] = []
{% if include_vector_search %}
    tools.append(build_retriever_tool())
{% endif %}
    # TODO: add additional tools here
    return tools
