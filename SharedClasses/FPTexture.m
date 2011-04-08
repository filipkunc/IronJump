//
//  FPTexture.m
//  SpriteLib
//
//  Created by Filip Kunc on 4/12/10.
//  For license see LICENSE.TXT
//

#import "FPTexture.h"

#if TARGET_OS_IPHONE

CGImageRef LoadImage(NSString *fileName)
{
	UIImage *image = [UIImage imageNamed:fileName];
	if (image == nil)
		@throw [NSException exceptionWithName:@"ImageNotLoaded" reason:fileName userInfo:nil];
	return [image CGImage];
}

#endif

void CreateTexture(GLubyte *data, int components, GLuint *textureID, int width, int height, BOOL convertToAlpha)
{
	glEnable(GL_TEXTURE_2D);
	glGenTextures(1, textureID);
	glBindTexture(GL_TEXTURE_2D, *textureID);
	
	if (convertToAlpha)
	{
		GLubyte *alphaData = (GLubyte *)malloc(width * height);
		for (int i = 0; i < width * height; i++)
			alphaData[i] = data[i * components];
		
		glTexImage2D(GL_TEXTURE_2D, 0, GL_ALPHA, width, height, 0, GL_ALPHA, GL_UNSIGNED_BYTE, alphaData);
		free(alphaData);
	}
	else 
	{
		if (components == 3)
			glTexImage2D(GL_TEXTURE_2D, 0, GL_RGB, width, height, 0, GL_RGB, GL_UNSIGNED_BYTE, data);
		else if (components == 4)
			glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA, width, height, 0, GL_RGBA, GL_UNSIGNED_BYTE, data);
		else
			@throw [NSException exceptionWithName:@"Unsupported image format" reason:nil userInfo:nil];

	}
	
	glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
	glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
	glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
	glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
}

@implementation FPTexture

@synthesize textureID, width, height;

#if !TARGET_OS_IPHONE

- (id)initWithFile:(NSString *)fileName convertToAlpha:(BOOL)convertToAlpha
{
	self = [super init];
	if (self)
	{
		NSImage *image = [NSImage imageNamed:fileName];
		width = [image size].width;
		height = [image size].height;
		
		glEnable(GL_TEXTURE_2D);
		glGenTextures(1, &textureID);
		glBindTexture(GL_TEXTURE_2D, textureID);
		
		NSBitmapImageRep *bitmap = [NSBitmapImageRep imageRepWithData:[image TIFFRepresentation]];
		GLubyte *data = [bitmap bitmapData];
		NSInteger bitsPerPixel = [bitmap bitsPerPixel];
		int components = bitsPerPixel / 8;
		 
		CreateTexture(data, components, &textureID, width, height, convertToAlpha);
		return self;
	}
	return nil;	
}
		 
#else

- (id)initWithFile:(NSString *)fileName convertToAlpha:(BOOL)convertToAlpha
{
	self = [super init];
	if (self)
	{
		CGImageRef image;
		CGContextRef context;
		GLubyte * data;
		
		image = LoadImage(fileName);
		width = CGImageGetWidth(image);
		height = CGImageGetHeight(image);
		
		if (image) 
		{
			data = (GLubyte *)malloc(width * height * 4);
			memset(data, 0, width * height * 4);
			
			context = CGBitmapContextCreate(data, width, height, 8, width * 4, CGImageGetColorSpace(image), kCGImageAlphaPremultipliedLast);
			CGContextDrawImage(context, CGRectMake(0.0f, 0.0f, (float)width, (float)height), image);
			CGContextRelease(context);
			
			CreateTexture(data, 4, &textureID, width, height, convertToAlpha);
			free(data);
			
			return self;
		}
	}
	return nil;
}

#endif

- (void)draw
{
	const FPVertex vertices[] = 
	{
		{ 0,	 0,			0, 0, }, // 0
		{ width, 0,			1, 0, }, // 1
		{ 0,	 height,	0, 1, }, // 2
		{ width, height,	1, 1, }, // 3
	};	
	
	glEnable(GL_TEXTURE_2D);
	glBindTexture(GL_TEXTURE_2D, textureID);
	glEnableClientState(GL_VERTEX_ARRAY);
	glVertexPointer(2, GL_FLOAT, sizeof(FPVertex), &vertices->x);	
	glEnableClientState(GL_TEXTURE_COORD_ARRAY);
	glTexCoordPointer(2, GL_SHORT, sizeof(FPVertex), &vertices->s);
	glDrawArrays(GL_TRIANGLE_STRIP, 0, 4);
	
}

- (void)drawAtPoint:(CGPoint)pt
{
#if TARGET_OS_IPHONE
	if (pt.x > 400.0f || pt.x + width < -80.0f)
		return;
	
	if (pt.y > 400.0f || pt.y + height < -80.0f)
		return;
#endif
	
	const FPVertex vertices[] = 
	{
		{ pt.x,			pt.y,			0, 0, }, // 0
		{ pt.x + width, pt.y,			1, 0, }, // 1
		{ pt.x,			pt.y + height,	0, 1, }, // 2
		{ pt.x + width, pt.y + height,	1, 1, }, // 3
	};	
	
	glEnable(GL_TEXTURE_2D);
	glBindTexture(GL_TEXTURE_2D, textureID);
	glEnableClientState(GL_VERTEX_ARRAY);
	glVertexPointer(2, GL_FLOAT, sizeof(FPVertex), &vertices->x);	
	glEnableClientState(GL_TEXTURE_COORD_ARRAY);
	glTexCoordPointer(2, GL_SHORT, sizeof(FPVertex), &vertices->s);
	glDrawArrays(GL_TRIANGLE_STRIP, 0, 4);
}

#if !TARGET_OS_IPHONE

- (void)drawAtPoint:(CGPoint)point widthSegments:(int)widthSegments heightSegments:(int)heightSegments
{
	for (int y = 0; y < heightSegments; y++)
	{
		for (int x = 0; x < widthSegments; x++)
		{
			CGPoint pt = CGPointMake(point.x + x * width, point.y + y * height);			
			[self drawAtPoint:pt];
		}
	}
}

#else

- (void)drawAtPoint:(CGPoint)point widthSegments:(int)widthSegments heightSegments:(int)heightSegments
{
	FPVertex *vertexBuffer = (FPVertex *)globalVertexBuffer;
	FPVertex *vertexPtr = vertexBuffer;
	int verticesUsed = 0;
	
	for (int y = 0; y < heightSegments; y++)
	{
		for (int x = 0; x < widthSegments; x++)
		{
			CGPoint pt = CGPointMake(point.x + x * width, point.y + y * height);
			
			if (pt.x > 400.0f || pt.x + width < -80.0f ||
				pt.y > 400.0f || pt.y + height < -80.0f)
				continue;
			
			/*
			 0     1
			 *-----*
			 |   / |
			 |  /  |
			 | /   |
			 *-----*
			 2     3
			*/
			
			const FPVertex vertices[] = 
			{
				{ pt.x,			pt.y,			0, 0, }, // 0
				{ pt.x + width, pt.y,			1, 0, }, // 1
				{ pt.x,			pt.y + height,	0, 1, }, // 2
				
				{ pt.x,			pt.y + height,	0, 1, }, // 2
				{ pt.x + width, pt.y + height,	1, 1, }, // 3
				{ pt.x + width, pt.y,			1, 0, }, // 1
			};
			
			verticesUsed += 6;
			if (verticesUsed > kMaxVertices)
			{
				@throw [NSException exceptionWithName:@"TextureDrawingFailed" reason:@"verticesUsed >= kMaxVertices" userInfo:nil];
			}
			
			memcpy(vertexPtr, vertices, sizeof(vertices));
			vertexPtr += 6;
		}
	}
	
	if (verticesUsed > 0)
	{
		glEnable(GL_TEXTURE_2D);
		glBindTexture(GL_TEXTURE_2D, textureID);
		glEnableClientState(GL_VERTEX_ARRAY);
		glVertexPointer(2, GL_FLOAT, sizeof(FPVertex), &vertexBuffer->x);	
		glEnableClientState(GL_TEXTURE_COORD_ARRAY);
		glTexCoordPointer(2, GL_SHORT, sizeof(FPVertex), &vertexBuffer->s);		
		glDrawArrays(GL_TRIANGLES, 0, verticesUsed);
	}
}

#endif

@end

